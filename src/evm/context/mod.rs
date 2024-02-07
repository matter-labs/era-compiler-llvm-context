//!
//! The LLVM IR generator context.
//!

pub mod address_space;
pub mod build;
pub mod debug_info;
pub mod evmla_data;
pub mod function;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use inkwell::types::BasicType;

use crate::context::attribute::Attribute;
use crate::context::code_type::CodeType;
use crate::context::function::declaration::Declaration as FunctionDeclaration;
use crate::context::function::r#return::Return as FunctionReturn;
use crate::context::r#loop::Loop;
use crate::context::IContext;
use crate::evm::DebugConfig;
use crate::evm::Dependency;
use crate::optimizer::Optimizer;
use crate::target_machine::target::Target;
use crate::target_machine::TargetMachine;

use self::address_space::AddressSpace;
use self::build::Build;
use self::debug_info::DebugInfo;
use self::evmla_data::EVMLAData;
use self::function::intrinsics::Intrinsics;
use self::function::Function;

///
/// The LLVM IR generator context.
///
/// It is a not-so-big god-like object glueing all the compilers' complexity and act as an adapter
/// and a superstructure over the inner `inkwell` LLVM context.
///
pub struct Context<'ctx, D>
where
    D: Dependency + Clone,
{
    /// The inner LLVM context.
    llvm: &'ctx inkwell::context::Context,
    /// The inner LLVM context builder.
    builder: inkwell::builder::Builder<'ctx>,
    /// The optimization tools.
    optimizer: Optimizer,
    /// The current module.
    module: inkwell::module::Module<'ctx>,
    /// The current contract code type, which can be deploy or runtime.
    code_type: CodeType,
    /// The LLVM intrinsic functions, defined on the LLVM side.
    intrinsics: Intrinsics<'ctx>,
    /// The declared functions.
    functions: HashMap<String, Rc<RefCell<Function<'ctx>>>>,
    /// The current active function.
    current_function: Option<Rc<RefCell<Function<'ctx>>>>,
    /// The loop context stack.
    loop_stack: Vec<Loop<'ctx>>,

    /// The project dependency manager. It can be any entity implementing the trait.
    /// The manager is used to get information about contracts and their dependencies during
    /// the multi-threaded compilation process.
    dependency_manager: Option<D>,
    /// Whether to append the metadata hash at the end of bytecode.
    include_metadata_hash: bool,
    /// The debug info of the current module.
    debug_info: DebugInfo<'ctx>,
    /// The debug configuration telling whether to dump the needed IRs.
    debug_config: Option<DebugConfig>,

    /// The EVM legacy assembly data.
    evmla_data: Option<EVMLAData<'ctx>>,
}

impl<'ctx, D> Context<'ctx, D>
where
    D: Dependency + Clone,
{
    /// The functions hashmap default capacity.
    const FUNCTIONS_HASHMAP_INITIAL_CAPACITY: usize = 64;

    /// The loop stack default capacity.
    const LOOP_STACK_INITIAL_CAPACITY: usize = 16;

    ///
    /// Initializes a new LLVM context.
    ///
    pub fn new(
        llvm: &'ctx inkwell::context::Context,
        module: inkwell::module::Module<'ctx>,
        code_type: CodeType,
        optimizer: Optimizer,
        dependency_manager: Option<D>,
        include_metadata_hash: bool,
        debug_config: Option<DebugConfig>,
    ) -> Self {
        let builder = llvm.create_builder();
        let intrinsics = Intrinsics::new(llvm, &module);
        let debug_info = DebugInfo::new(&module);

        Self {
            llvm,
            builder,
            optimizer,
            module,
            code_type,
            intrinsics,
            functions: HashMap::with_capacity(Self::FUNCTIONS_HASHMAP_INITIAL_CAPACITY),
            current_function: None,
            loop_stack: Vec::with_capacity(Self::LOOP_STACK_INITIAL_CAPACITY),

            dependency_manager,
            include_metadata_hash,
            debug_info,
            debug_config,

            evmla_data: None,
        }
    }

    ///
    /// Builds the LLVM IR module, returning the build artifacts.
    ///
    pub fn build(
        self,
        contract_path: &str,
        metadata_hash: Option<[u8; era_compiler_common::BYTE_LENGTH_FIELD]>,
    ) -> anyhow::Result<Build> {
        let target_machine = TargetMachine::new(Target::EVM, self.optimizer.settings())?;
        target_machine.set_target_data(self.module());

        if let Some(ref debug_config) = self.debug_config {
            debug_config.dump_llvm_ir_unoptimized(
                contract_path,
                Some(self.code_type),
                self.module(),
            )?;
        }
        self.verify().map_err(|error| {
            anyhow::anyhow!(
                "The contract `{}` {} code unoptimized LLVM IR verification error: {}",
                contract_path,
                self.code_type,
                error
            )
        })?;

        self.optimizer
            .run(&target_machine, self.module())
            .map_err(|error| {
                anyhow::anyhow!(
                    "The contract `{}` {} code optimizing error: {}",
                    contract_path,
                    self.code_type,
                    error
                )
            })?;
        if let Some(ref debug_config) = self.debug_config {
            debug_config.dump_llvm_ir_optimized(
                contract_path,
                Some(self.code_type),
                self.module(),
            )?;
        }
        self.verify().map_err(|error| {
            anyhow::anyhow!(
                "The contract `{}` {} code optimized LLVM IR verification error: {}",
                contract_path,
                self.code_type,
                error
            )
        })?;

        let buffer = target_machine
            .write_to_memory_buffer(self.module())
            .map_err(|error| {
                anyhow::anyhow!(
                    "The contract `{}` {} code assembly generating error: {}",
                    contract_path,
                    self.code_type,
                    error
                )
            })?;

        Ok(Build::new(
            String::new(),
            metadata_hash,
            buffer.as_slice().to_vec(),
        ))
    }

    ///
    /// Verifies the current LLVM IR module.
    ///
    pub fn verify(&self) -> anyhow::Result<()> {
        self.module()
            .verify()
            .map_err(|error| anyhow::anyhow!(error.to_string()))
    }

    ///
    /// Returns the LLVM intrinsics collection reference.
    ///
    pub fn intrinsics(&self) -> &Intrinsics<'ctx> {
        &self.intrinsics
    }

    ///
    /// Compiles a contract dependency, if the dependency manager is set.
    ///
    pub fn compile_dependency(&mut self, name: &str) -> anyhow::Result<String> {
        self.dependency_manager
            .to_owned()
            .ok_or_else(|| anyhow::anyhow!("The dependency manager is unset"))
            .and_then(|manager| {
                Dependency::compile(
                    manager,
                    name,
                    self.optimizer.settings().to_owned(),
                    self.include_metadata_hash,
                    self.debug_config.clone(),
                )
            })
    }

    ///
    /// Gets a full contract_path from the dependency manager.
    ///
    pub fn resolve_path(&self, identifier: &str) -> anyhow::Result<String> {
        self.dependency_manager
            .to_owned()
            .ok_or_else(|| anyhow::anyhow!("The dependency manager is unset"))
            .and_then(|manager| {
                let full_path = manager.resolve_path(identifier)?;
                Ok(full_path)
            })
    }

    ///
    /// Gets a deployed library address from the dependency manager.
    ///
    pub fn resolve_library(&self, path: &str) -> anyhow::Result<inkwell::values::IntValue<'ctx>> {
        self.dependency_manager
            .to_owned()
            .ok_or_else(|| anyhow::anyhow!("The dependency manager is unset"))
            .and_then(|manager| {
                let address = manager.resolve_library(path)?;
                let address = self.field_const_str_hex(address.as_str());
                Ok(address)
            })
    }

    ///
    /// Extracts the dependency manager.
    ///
    pub fn take_dependency_manager(&mut self) -> D {
        self.dependency_manager
            .take()
            .expect("The dependency manager is unset")
    }

    ///
    /// Returns the debug info reference.
    ///
    pub fn debug_info(&self) -> &DebugInfo<'ctx> {
        &self.debug_info
    }

    ///
    /// Returns the debug config reference.
    ///
    pub fn debug_config(&self) -> Option<&DebugConfig> {
        self.debug_config.as_ref()
    }

    ///
    /// Returns a Yul function type with the specified arguments and number of return values.
    ///
    pub fn function_type<T>(
        &self,
        argument_types: Vec<T>,
        return_values_size: usize,
    ) -> inkwell::types::FunctionType<'ctx>
    where
        T: BasicType<'ctx>,
    {
        let argument_types: Vec<inkwell::types::BasicMetadataTypeEnum> = argument_types
            .as_slice()
            .iter()
            .map(T::as_basic_type_enum)
            .map(inkwell::types::BasicMetadataTypeEnum::from)
            .collect();
        match return_values_size {
            0 => self
                .llvm
                .void_type()
                .fn_type(argument_types.as_slice(), false),
            1 => self.field_type().fn_type(argument_types.as_slice(), false),
            size => self
                .structure_type(vec![self.field_type().as_basic_type_enum(); size].as_slice())
                .fn_type(argument_types.as_slice(), false),
        }
    }

    ///
    /// Modifies the call site value, setting the default attributes.
    ///
    /// The attributes only affect the LLVM optimizations.
    ///
    pub fn modify_call_site_value(
        &self,
        arguments: &[inkwell::values::BasicValueEnum<'ctx>],
        call_site_value: inkwell::values::CallSiteValue<'ctx>,
        function: FunctionDeclaration<'ctx>,
    ) {
        for (index, argument) in arguments.iter().enumerate() {
            if argument.is_pointer_value() {
                call_site_value.set_alignment_attribute(
                    inkwell::attributes::AttributeLoc::Param(index as u32),
                    era_compiler_common::BYTE_LENGTH_FIELD as u32,
                );
                call_site_value.add_attribute(
                    inkwell::attributes::AttributeLoc::Param(index as u32),
                    self.llvm
                        .create_enum_attribute(Attribute::NoAlias as u32, 0),
                );
                call_site_value.add_attribute(
                    inkwell::attributes::AttributeLoc::Param(index as u32),
                    self.llvm
                        .create_enum_attribute(Attribute::NoCapture as u32, 0),
                );
                call_site_value.add_attribute(
                    inkwell::attributes::AttributeLoc::Param(index as u32),
                    self.llvm.create_enum_attribute(Attribute::NoFree as u32, 0),
                );
                if Some(argument.get_type()) == function.r#type.get_return_type() {
                    if function
                        .r#type
                        .get_return_type()
                        .map(|r#type| {
                            r#type.into_pointer_type().get_address_space()
                                == AddressSpace::Stack.into()
                        })
                        .unwrap_or_default()
                    {
                        call_site_value.add_attribute(
                            inkwell::attributes::AttributeLoc::Param(index as u32),
                            self.llvm
                                .create_enum_attribute(Attribute::Returned as u32, 0),
                        );
                    }
                    call_site_value.add_attribute(
                        inkwell::attributes::AttributeLoc::Param(index as u32),
                        self.llvm.create_enum_attribute(
                            Attribute::Dereferenceable as u32,
                            (era_compiler_common::BIT_LENGTH_FIELD * 2) as u64,
                        ),
                    );
                    call_site_value.add_attribute(
                        inkwell::attributes::AttributeLoc::Return,
                        self.llvm.create_enum_attribute(
                            Attribute::Dereferenceable as u32,
                            (era_compiler_common::BIT_LENGTH_FIELD * 2) as u64,
                        ),
                    );
                }
                call_site_value.add_attribute(
                    inkwell::attributes::AttributeLoc::Param(index as u32),
                    self.llvm
                        .create_enum_attribute(Attribute::NonNull as u32, 0),
                );
                call_site_value.add_attribute(
                    inkwell::attributes::AttributeLoc::Param(index as u32),
                    self.llvm
                        .create_enum_attribute(Attribute::NoUndef as u32, 0),
                );
            }
        }

        if function
            .r#type
            .get_return_type()
            .map(|r#type| r#type.is_pointer_type())
            .unwrap_or_default()
        {
            call_site_value.set_alignment_attribute(
                inkwell::attributes::AttributeLoc::Return,
                era_compiler_common::BYTE_LENGTH_FIELD as u32,
            );
            call_site_value.add_attribute(
                inkwell::attributes::AttributeLoc::Return,
                self.llvm
                    .create_enum_attribute(Attribute::NoAlias as u32, 0),
            );
            call_site_value.add_attribute(
                inkwell::attributes::AttributeLoc::Return,
                self.llvm
                    .create_enum_attribute(Attribute::NonNull as u32, 0),
            );
            call_site_value.add_attribute(
                inkwell::attributes::AttributeLoc::Return,
                self.llvm
                    .create_enum_attribute(Attribute::NoUndef as u32, 0),
            );
        }
    }
}

impl<'ctx, D> IContext<'ctx> for Context<'ctx, D>
where
    D: Dependency + Clone,
{
    type Function = Function<'ctx>;

    type AddressSpace = AddressSpace;

    type SolidityData = ();

    type YulData = ();

    type EVMLAData = EVMLAData<'ctx>;

    type VyperData = ();

    fn llvm(&self) -> &'ctx inkwell::context::Context {
        self.llvm
    }

    fn builder(&self) -> &inkwell::builder::Builder<'ctx> {
        &self.builder
    }

    fn module(&self) -> &inkwell::module::Module<'ctx> {
        &self.module
    }

    fn set_code_type(&mut self, code_type: CodeType) {
        self.code_type = code_type;
    }

    fn code_type(&self) -> Option<CodeType> {
        Some(self.code_type.to_owned())
    }

    fn append_basic_block(&self, name: &str) -> inkwell::basic_block::BasicBlock<'ctx> {
        self.llvm()
            .append_basic_block(self.current_function().borrow().declaration().value, name)
    }

    fn set_basic_block(&self, block: inkwell::basic_block::BasicBlock<'ctx>) {
        self.builder().position_at_end(block);
    }

    fn basic_block(&self) -> inkwell::basic_block::BasicBlock<'ctx> {
        self.builder().get_insert_block().expect("Always exists")
    }

    fn push_loop(
        &mut self,
        body_block: inkwell::basic_block::BasicBlock<'ctx>,
        continue_block: inkwell::basic_block::BasicBlock<'ctx>,
        join_block: inkwell::basic_block::BasicBlock<'ctx>,
    ) {
        self.loop_stack
            .push(Loop::new(body_block, continue_block, join_block));
    }

    fn pop_loop(&mut self) {
        self.loop_stack.pop();
    }

    fn r#loop(&self) -> &Loop<'ctx> {
        self.loop_stack
            .last()
            .expect("The current context is not in a loop")
    }

    fn add_function(
        &mut self,
        name: &str,
        r#type: inkwell::types::FunctionType<'ctx>,
        return_values_length: usize,
        linkage: Option<inkwell::module::Linkage>,
    ) -> anyhow::Result<Rc<RefCell<Self::Function>>> {
        let value = self.module().add_function(name, r#type, linkage);

        let entry_block = self.llvm.append_basic_block(value, "entry");
        let return_block = self.llvm.append_basic_block(value, "return");

        let r#return = match return_values_length {
            0 => FunctionReturn::none(),
            1 => {
                self.set_basic_block(entry_block);
                let pointer = self.build_alloca(self.field_type(), "return_pointer");
                FunctionReturn::primitive(pointer)
            }
            size => {
                self.set_basic_block(entry_block);
                let pointer = self.build_alloca(
                    self.structure_type(
                        vec![self.field_type().as_basic_type_enum(); size].as_slice(),
                    ),
                    "return_pointer",
                );
                FunctionReturn::compound(pointer, size)
            }
        };

        let function = Function::new(
            name.to_owned(),
            FunctionDeclaration::new(r#type, value),
            r#return,
            entry_block,
            return_block,
        );
        Function::set_default_attributes(self.llvm, function.declaration(), &self.optimizer);

        let function = Rc::new(RefCell::new(function));
        self.functions.insert(name.to_string(), function.clone());

        Ok(function)
    }

    fn get_function(&self, name: &str) -> Option<Rc<RefCell<Self::Function>>> {
        self.functions.get(name).cloned()
    }

    fn current_function(&self) -> Rc<RefCell<Self::Function>> {
        self.current_function
            .clone()
            .expect("Must be declared before use")
    }

    fn set_current_function(&mut self, name: &str) -> anyhow::Result<()> {
        let function = self.functions.get(name).cloned().ok_or_else(|| {
            anyhow::anyhow!("Failed to activate an undeclared function `{}`", name)
        })?;
        self.current_function = Some(function);
        Ok(())
    }

    fn build_call(
        &self,
        function: FunctionDeclaration<'ctx>,
        arguments: &[inkwell::values::BasicValueEnum<'ctx>],
        name: &str,
    ) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
        let arguments_wrapped: Vec<inkwell::values::BasicMetadataValueEnum> = arguments
            .iter()
            .copied()
            .map(inkwell::values::BasicMetadataValueEnum::from)
            .collect();
        let call_site_value = self.builder.build_indirect_call(
            function.r#type,
            function.value.as_global_value().as_pointer_value(),
            arguments_wrapped.as_slice(),
            name,
        );
        self.modify_call_site_value(arguments, call_site_value, function);
        call_site_value.try_as_basic_value().left()
    }

    fn build_invoke(
        &self,
        function: FunctionDeclaration<'ctx>,
        arguments: &[inkwell::values::BasicValueEnum<'ctx>],
        name: &str,
    ) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
        Self::build_call(self, function, arguments, name)
    }

    fn set_solidity_data(&mut self, _data: Self::SolidityData) {
        panic!("Unused with the EVM target");
    }

    fn solidity(&self) -> &Self::SolidityData {
        panic!("Unused with the EVM target");
    }

    fn solidity_mut(&mut self) -> &mut Self::SolidityData {
        panic!("Unused with the EVM target");
    }

    fn set_yul_data(&mut self, _data: Self::YulData) {
        panic!("Unused with the EVM target");
    }

    fn yul(&self) -> &Self::YulData {
        panic!("Unused with the EVM target");
    }

    fn yul_mut(&mut self) -> &mut Self::YulData {
        panic!("Unused with the EVM target");
    }

    fn set_evmla_data(&mut self, data: Self::EVMLAData) {
        self.evmla_data = Some(data);
    }

    fn evmla(&self) -> &Self::EVMLAData {
        self.evmla_data
            .as_ref()
            .expect("The EVMLA data must have been initialized")
    }

    fn evmla_mut(&mut self) -> &mut Self::EVMLAData {
        self.evmla_data
            .as_mut()
            .expect("The EVMLA data must have been initialized")
    }

    fn set_vyper_data(&mut self, _data: Self::VyperData) {
        panic!("Unused with the EVM target");
    }

    fn vyper(&self) -> &Self::VyperData {
        panic!("Unused with the EVM target");
    }
}
