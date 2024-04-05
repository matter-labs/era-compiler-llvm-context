//!
//! The LLVM IR generator function.
//!

pub mod intrinsics;
pub mod llvm_runtime;
pub mod runtime;
pub mod vyper_data;
pub mod yul_data;

use std::collections::HashMap;

use crate::context::attribute::memory::Memory as MemoryAttribute;
use crate::context::attribute::Attribute;
use crate::context::function::block::key::Key as BlockKey;
use crate::context::function::block::Block;
use crate::context::function::declaration::Declaration as FunctionDeclaration;
use crate::context::function::evmla_data::EVMLAData as FunctionEVMLAData;
use crate::context::function::r#return::Return as FunctionReturn;
use crate::context::pointer::Pointer;
use crate::context::traits::evmla_function::IEVMLAFunction;
use crate::eravm::context::address_space::AddressSpace;
use crate::optimizer::settings::size_level::SizeLevel;
use crate::optimizer::Optimizer;

use self::runtime::Runtime;
use self::vyper_data::VyperData;
use self::yul_data::YulData;

///
/// The LLVM IR generator function.
///
#[derive(Debug)]
pub struct Function<'ctx> {
    /// The high-level source code name.
    name: String,
    /// The LLVM function declaration.
    declaration: FunctionDeclaration<'ctx>,
    /// The stack representation.
    stack: HashMap<String, Pointer<'ctx, AddressSpace>>,
    /// The return value entity.
    r#return: FunctionReturn<'ctx, AddressSpace>,

    /// The entry block. Each LLVM IR functions must have an entry block.
    entry_block: inkwell::basic_block::BasicBlock<'ctx>,
    /// The return/leave block. LLVM IR functions may have multiple returning blocks, but it is
    /// more reasonable to have a single returning block and other high-level language returns
    /// jumping to it. This way it is easier to implement some additional checks and clean-ups
    /// before the returning.
    return_block: inkwell::basic_block::BasicBlock<'ctx>,

    /// The Yul compiler data.
    yul_data: Option<YulData>,
    /// The EVM legacy assembly compiler data.
    evmla_data: Option<FunctionEVMLAData<'ctx>>,
    /// The Vyper data.
    vyper_data: Option<VyperData>,
}

impl<'ctx> Function<'ctx> {
    /// The near call ABI function prefix.
    pub const ZKSYNC_NEAR_CALL_ABI_PREFIX: &'static str = "ZKSYNC_NEAR_CALL";

    /// The near call ABI exception handler name.
    pub const ZKSYNC_NEAR_CALL_ABI_EXCEPTION_HANDLER: &'static str = "ZKSYNC_CATCH_NEAR_CALL";

    /// The stack hashmap default capacity.
    const STACK_HASHMAP_INITIAL_CAPACITY: usize = 64;

    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        name: String,
        declaration: FunctionDeclaration<'ctx>,
        r#return: FunctionReturn<'ctx, AddressSpace>,

        entry_block: inkwell::basic_block::BasicBlock<'ctx>,
        return_block: inkwell::basic_block::BasicBlock<'ctx>,
    ) -> Self {
        Self {
            name,
            declaration,
            stack: HashMap::with_capacity(Self::STACK_HASHMAP_INITIAL_CAPACITY),
            r#return,

            entry_block,
            return_block,

            yul_data: None,
            evmla_data: None,
            vyper_data: None,
        }
    }

    ///
    /// Returns the function name reference.
    ///
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    ///
    /// Checks whether the function is defined outside of the front-end.
    ///
    pub fn is_name_external(name: &str) -> bool {
        name.starts_with("llvm.")
            || (name.starts_with("__")
                && name != Runtime::FUNCTION_ENTRY
                && name != Runtime::FUNCTION_DEPLOY_CODE
                && name != Runtime::FUNCTION_RUNTIME_CODE)
    }

    ///
    /// Checks whether the function is related to the near call ABI.
    ///
    pub fn is_near_call_abi(name: &str) -> bool {
        name.starts_with(Self::ZKSYNC_NEAR_CALL_ABI_PREFIX)
            || name == Self::ZKSYNC_NEAR_CALL_ABI_EXCEPTION_HANDLER
    }

    ///
    /// Returns the LLVM function declaration.
    ///
    pub fn declaration(&self) -> FunctionDeclaration<'ctx> {
        self.declaration
    }

    ///
    /// Returns the N-th parameter of the function.
    ///
    pub fn get_nth_param(&self, index: usize) -> inkwell::values::BasicValueEnum<'ctx> {
        self.declaration()
            .value
            .get_nth_param(index as u32)
            .expect("Always exists")
    }

    ///
    /// Sets the memory writer function attributes.
    ///
    pub fn set_attributes(
        llvm: &'ctx inkwell::context::Context,
        declaration: FunctionDeclaration<'ctx>,
        attributes: Vec<(Attribute, Option<u64>)>,
        force: bool,
    ) {
        for (attribute_kind, value) in attributes.into_iter() {
            match attribute_kind {
                attribute_kind @ Attribute::AlwaysInline if force => {
                    let is_optimize_none_set = declaration
                        .value
                        .get_enum_attribute(
                            inkwell::attributes::AttributeLoc::Function,
                            Attribute::OptimizeNone as u32,
                        )
                        .is_some();
                    if !is_optimize_none_set {
                        declaration.value.remove_enum_attribute(
                            inkwell::attributes::AttributeLoc::Function,
                            Attribute::NoInline as u32,
                        );
                        declaration.value.add_attribute(
                            inkwell::attributes::AttributeLoc::Function,
                            llvm.create_enum_attribute(attribute_kind as u32, 0),
                        );
                    }
                }
                attribute_kind @ Attribute::NoInline if force => {
                    declaration.value.remove_enum_attribute(
                        inkwell::attributes::AttributeLoc::Function,
                        Attribute::AlwaysInline as u32,
                    );
                    declaration.value.add_attribute(
                        inkwell::attributes::AttributeLoc::Function,
                        llvm.create_enum_attribute(attribute_kind as u32, 0),
                    );
                }
                attribute_kind @ Attribute::Memory => {
                    declaration.value.add_attribute(
                        inkwell::attributes::AttributeLoc::Function,
                        llvm.create_enum_attribute(
                            attribute_kind as u32,
                            value.expect("The `memory` attribute always requires a value"),
                        ),
                    );
                }
                attribute_kind => declaration.value.add_attribute(
                    inkwell::attributes::AttributeLoc::Function,
                    llvm.create_enum_attribute(attribute_kind as u32, 0),
                ),
            }
        }
    }

    ///
    /// Sets the default attributes.
    ///
    /// The attributes only affect the LLVM optimizations.
    ///
    pub fn set_default_attributes(
        llvm: &'ctx inkwell::context::Context,
        declaration: FunctionDeclaration<'ctx>,
        optimizer: &Optimizer,
    ) {
        if optimizer.settings().level_middle_end == inkwell::OptimizationLevel::None {
            Self::set_attributes(
                llvm,
                declaration,
                vec![(Attribute::OptimizeNone, None), (Attribute::NoInline, None)],
                false,
            );
        } else if optimizer.settings().level_middle_end_size == SizeLevel::Z {
            Self::set_attributes(
                llvm,
                declaration,
                vec![
                    (Attribute::OptimizeForSize, None),
                    (Attribute::MinSize, None),
                ],
                false,
            );
        }

        Self::set_attributes(
            llvm,
            declaration,
            vec![
                (Attribute::NoFree, None),
                (Attribute::NullPointerIsValid, None),
            ],
            false,
        );
    }

    ///
    /// Sets the front-end runtime attributes.
    ///
    pub fn set_frontend_runtime_attributes(
        llvm: &'ctx inkwell::context::Context,
        declaration: FunctionDeclaration<'ctx>,
        optimizer: &Optimizer,
    ) {
        if optimizer.settings().level_middle_end_size == SizeLevel::Z {
            Self::set_attributes(llvm, declaration, vec![(Attribute::NoInline, None)], false);
        }
    }

    ///
    /// Sets the exception handler attributes.
    ///
    pub fn set_exception_handler_attributes(
        llvm: &'ctx inkwell::context::Context,
        declaration: FunctionDeclaration<'ctx>,
    ) {
        Self::set_attributes(llvm, declaration, vec![(Attribute::NoInline, None)], false);
    }

    ///
    /// Sets the CXA-throw attributes.
    ///
    pub fn set_cxa_throw_attributes(
        llvm: &'ctx inkwell::context::Context,
        declaration: FunctionDeclaration<'ctx>,
    ) {
        Self::set_attributes(llvm, declaration, vec![(Attribute::NoProfile, None)], false);
    }

    ///
    /// Sets the pure function attributes.
    ///
    pub fn set_pure_function_attributes(
        llvm: &'ctx inkwell::context::Context,
        declaration: FunctionDeclaration<'ctx>,
    ) {
        Self::set_attributes(
            llvm,
            declaration,
            vec![
                (Attribute::MustProgress, None),
                (Attribute::NoUnwind, None),
                (Attribute::WillReturn, None),
                // (Attribute::Memory, Some(MemoryAttribute::None as u64)),
            ],
            false,
        );
    }

    ///
    /// Saves the pointer to a stack variable, returning the pointer to the shadowed variable,
    /// if it exists.
    ///
    pub fn insert_stack_pointer(
        &mut self,
        name: String,
        pointer: Pointer<'ctx, AddressSpace>,
    ) -> Option<Pointer<'ctx, AddressSpace>> {
        self.stack.insert(name, pointer)
    }

    ///
    /// Gets the pointer to a stack variable.
    ///
    pub fn get_stack_pointer(&self, name: &str) -> Option<Pointer<'ctx, AddressSpace>> {
        self.stack.get(name).copied()
    }

    ///
    /// Removes the pointer to a stack variable.
    ///
    pub fn remove_stack_pointer(&mut self, name: &str) {
        self.stack.remove(name);
    }

    ///
    /// Returns the return entity representation.
    ///
    pub fn r#return(&self) -> FunctionReturn<'ctx, AddressSpace> {
        self.r#return
    }

    ///
    /// Returns the pointer to the function return value.
    ///
    /// # Panics
    /// If the pointer has not been set yet.
    ///
    pub fn return_pointer(&self) -> Option<Pointer<'ctx, AddressSpace>> {
        self.r#return.return_pointer()
    }

    ///
    /// Returns the return data size in bytes, based on the default stack alignment.
    ///
    /// # Panics
    /// If the pointer has not been set yet.
    ///
    pub fn return_data_size(&self) -> usize {
        self.r#return.return_data_size()
    }

    ///
    /// Returns the function entry block.
    ///
    pub fn entry_block(&self) -> inkwell::basic_block::BasicBlock<'ctx> {
        self.entry_block
    }

    ///
    /// Returns the function return block.
    ///
    pub fn return_block(&self) -> inkwell::basic_block::BasicBlock<'ctx> {
        self.return_block
    }

    ///
    /// Sets the EVM legacy assembly data.
    ///
    pub fn set_evmla_data(&mut self, data: FunctionEVMLAData<'ctx>) {
        self.evmla_data = Some(data);
    }

    ///
    /// Returns the EVM legacy assembly data reference.
    ///
    /// # Panics
    /// If the EVM data has not been initialized.
    ///
    pub fn evmla(&self) -> &FunctionEVMLAData<'ctx> {
        self.evmla_data
            .as_ref()
            .expect("The EVM data must have been initialized")
    }

    ///
    /// Returns the EVM legacy assembly data mutable reference.
    ///
    /// # Panics
    /// If the EVM data has not been initialized.
    ///
    pub fn evmla_mut(&mut self) -> &mut FunctionEVMLAData<'ctx> {
        self.evmla_data
            .as_mut()
            .expect("The EVM data must have been initialized")
    }

    ///
    /// Sets the Vyper data.
    ///
    pub fn set_vyper_data(&mut self, data: VyperData) {
        self.vyper_data = Some(data);
    }

    ///
    /// Returns the Vyper data reference.
    ///
    /// # Panics
    /// If the Vyper data has not been initialized.
    ///
    pub fn vyper(&self) -> &VyperData {
        self.vyper_data
            .as_ref()
            .expect("The Vyper data must have been initialized")
    }

    ///
    /// Returns the Vyper data mutable reference.
    ///
    /// # Panics
    /// If the Vyper data has not been initialized.
    ///
    pub fn vyper_mut(&mut self) -> &mut VyperData {
        self.vyper_data
            .as_mut()
            .expect("The Vyper data must have been initialized")
    }

    ///
    /// Sets the Yul data.
    ///
    pub fn set_yul_data(&mut self, data: YulData) {
        self.yul_data = Some(data);
    }

    ///
    /// Returns the Yul data reference.
    ///
    /// # Panics
    /// If the Yul data has not been initialized.
    ///
    pub fn yul(&self) -> &YulData {
        self.yul_data
            .as_ref()
            .expect("The Yul data must have been initialized")
    }

    ///
    /// Returns the Yul data mutable reference.
    ///
    /// # Panics
    /// If the Yul data has not been initialized.
    ///
    pub fn yul_mut(&mut self) -> &mut YulData {
        self.yul_data
            .as_mut()
            .expect("The Yul data must have been initialized")
    }
}

impl<'ctx> IEVMLAFunction<'ctx> for Function<'ctx> {
    fn find_block(&self, key: &BlockKey, stack_hash: &md5::Digest) -> anyhow::Result<Block<'ctx>> {
        let evmla_data = self.evmla();

        if evmla_data
            .blocks
            .get(key)
            .ok_or_else(|| anyhow::anyhow!("Undeclared function block {}", key))?
            .len()
            == 1
        {
            return evmla_data
                .blocks
                .get(key)
                .ok_or_else(|| anyhow::anyhow!("Undeclared function block {}", key))?
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Undeclared function block {}", key));
        }

        evmla_data
            .blocks
            .get(key)
            .ok_or_else(|| anyhow::anyhow!("Undeclared function block {}", key))?
            .iter()
            .find(|block| {
                block
                    .evm()
                    .stack_hashes
                    .iter()
                    .any(|hash| hash == stack_hash)
            })
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Undeclared function block {}", key))
    }
}
