//!
//! The LLVM IR generator function.
//!

pub mod block;
pub mod declaration;
pub mod evmla_data;
pub mod intrinsics;
pub mod r#return;
pub mod runtime;
pub mod vyper_data;

use std::collections::HashMap;

use crate::attribute::Attribute;
use crate::evm::context::pointer::Pointer;
use crate::optimizer::settings::size_level::SizeLevel;
use crate::optimizer::Optimizer;

use self::declaration::Declaration;
use self::evmla_data::EVMLAData;
use self::r#return::Return;
use self::vyper_data::VyperData;

///
/// The LLVM IR generator function.
///
#[derive(Debug)]
pub struct Function<'ctx> {
    /// The high-level source code name.
    name: String,
    /// The LLVM function declaration.
    declaration: Declaration<'ctx>,
    /// The stack representation.
    stack: HashMap<String, Pointer<'ctx>>,
    /// The return value entity.
    r#return: Return<'ctx>,

    /// The entry block. Each LLVM IR functions must have an entry block.
    entry_block: inkwell::basic_block::BasicBlock<'ctx>,
    /// The return/leave block. LLVM IR functions may have multiple returning blocks, but it is
    /// more reasonable to have a single returning block and other high-level language returns
    /// jumping to it. This way it is easier to implement some additional checks and clean-ups
    /// before the returning.
    return_block: inkwell::basic_block::BasicBlock<'ctx>,

    /// The EVM legacy assembly compiler data.
    evmla_data: Option<EVMLAData<'ctx>>,
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
        declaration: Declaration<'ctx>,
        r#return: Return<'ctx>,

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
            || (name.starts_with("__") && name != crate::evm_const::ENTRY_FUNCTION_NAME)
    }

    ///
    /// Returns the LLVM function declaration.
    ///
    pub fn declaration(&self) -> Declaration<'ctx> {
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
    /// Sets the default attributes.
    ///
    /// The attributes only affect the LLVM optimizations.
    ///
    pub fn set_default_attributes(
        llvm: &'ctx inkwell::context::Context,
        declaration: Declaration<'ctx>,
        optimizer: &Optimizer,
    ) {
        if optimizer.settings().level_middle_end_size == SizeLevel::Z {
            declaration.value.add_attribute(
                inkwell::attributes::AttributeLoc::Function,
                llvm.create_enum_attribute(Attribute::OptimizeForSize as u32, 0),
            );
            declaration.value.add_attribute(
                inkwell::attributes::AttributeLoc::Function,
                llvm.create_enum_attribute(Attribute::MinSize as u32, 0),
            );
        }

        declaration.value.add_attribute(
            inkwell::attributes::AttributeLoc::Function,
            llvm.create_enum_attribute(Attribute::NoFree as u32, 0),
        );
        declaration.value.add_attribute(
            inkwell::attributes::AttributeLoc::Function,
            llvm.create_enum_attribute(Attribute::NullPointerIsValid as u32, 0),
        );
    }

    ///
    /// Sets the front-end runtime attributes.
    ///
    pub fn set_frontend_runtime_attributes(
        llvm: &'ctx inkwell::context::Context,
        declaration: Declaration<'ctx>,
        optimizer: &Optimizer,
    ) {
        if optimizer.settings().level_middle_end_size == SizeLevel::Z {
            declaration.value.add_attribute(
                inkwell::attributes::AttributeLoc::Function,
                llvm.create_enum_attribute(Attribute::NoInline as u32, 0),
            );
        }
    }

    ///
    /// Sets the exception handler attributes.
    ///
    pub fn set_exception_handler_attributes(
        llvm: &'ctx inkwell::context::Context,
        declaration: Declaration<'ctx>,
    ) {
        declaration.value.add_attribute(
            inkwell::attributes::AttributeLoc::Function,
            llvm.create_enum_attribute(Attribute::NoInline as u32, 0),
        );
    }

    ///
    /// Sets the LLVM runtime attributes.
    ///
    pub fn set_llvm_runtime_attributes(
        llvm: &'ctx inkwell::context::Context,
        declaration: Declaration<'ctx>,
    ) {
        for attribute_kind in [
            Attribute::MustProgress,
            Attribute::NoUnwind,
            Attribute::ReadNone,
            Attribute::WillReturn,
        ]
        .into_iter()
        {
            declaration.value.add_attribute(
                inkwell::attributes::AttributeLoc::Function,
                llvm.create_enum_attribute(attribute_kind as u32, 0),
            );
        }
    }

    ///
    /// Saves the pointer to a stack variable, returning the pointer to the shadowed variable,
    /// if it exists.
    ///
    pub fn insert_stack_pointer(
        &mut self,
        name: String,
        pointer: Pointer<'ctx>,
    ) -> Option<Pointer<'ctx>> {
        self.stack.insert(name, pointer)
    }

    ///
    /// Gets the pointer to a stack variable.
    ///
    pub fn get_stack_pointer(&self, name: &str) -> Option<Pointer<'ctx>> {
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
    pub fn r#return(&self) -> Return<'ctx> {
        self.r#return
    }

    ///
    /// Returns the pointer to the function return value.
    ///
    /// # Panics
    /// If the pointer has not been set yet.
    ///
    pub fn return_pointer(&self) -> Option<Pointer<'ctx>> {
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
    pub fn set_evmla_data(&mut self, data: EVMLAData<'ctx>) {
        self.evmla_data = Some(data);
    }

    ///
    /// Returns the EVM legacy assembly data reference.
    ///
    /// # Panics
    /// If the EVM data has not been initialized.
    ///
    pub fn evmla(&self) -> &EVMLAData<'ctx> {
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
    pub fn evmla_mut(&mut self) -> &mut EVMLAData<'ctx> {
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
}