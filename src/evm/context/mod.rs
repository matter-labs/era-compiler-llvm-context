//!
//! The LLVM IR generator context.
//!

pub mod address_space;
pub mod build;
pub mod debug_info;
pub mod evmla_data;
pub mod function;
pub mod r#loop;
pub mod pointer;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use inkwell::types::BasicType;
use inkwell::values::BasicValue;

use crate::attribute::Attribute;
use crate::code_type::CodeType;
use crate::evm::DebugConfig;
use crate::evm::Dependency;
use crate::optimizer::Optimizer;
use crate::target_machine::target::Target;
use crate::target_machine::TargetMachine;

use self::address_space::AddressSpace;
use self::build::Build;
use self::debug_info::DebugInfo;
use self::evmla_data::EVMLAData;
use self::function::declaration::Declaration as FunctionDeclaration;
use self::function::intrinsics::Intrinsics;
use self::function::r#return::Return as FunctionReturn;
use self::function::Function;
use self::pointer::Pointer;
use self::r#loop::Loop;

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
    /// Returns the LLVM IR builder.
    ///
    pub fn builder(&self) -> &inkwell::builder::Builder<'ctx> {
        &self.builder
    }

    ///
    /// Returns the current LLVM IR module reference.
    ///
    pub fn module(&self) -> &inkwell::module::Module<'ctx> {
        &self.module
    }

    ///
    /// Returns the LLVM intrinsics collection reference.
    ///
    pub fn intrinsics(&self) -> &Intrinsics<'ctx> {
        &self.intrinsics
    }

    ///
    /// Appends a function to the current module.
    ///
    pub fn add_function(
        &mut self,
        name: &str,
        r#type: inkwell::types::FunctionType<'ctx>,
        return_values_length: usize,
        linkage: Option<inkwell::module::Linkage>,
    ) -> anyhow::Result<Rc<RefCell<Function<'ctx>>>> {
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

    ///
    /// Returns a shared reference to the specified function.
    ///
    pub fn get_function(&self, name: &str) -> Option<Rc<RefCell<Function<'ctx>>>> {
        self.functions.get(name).cloned()
    }

    ///
    /// Returns a shared reference to the current active function.
    ///
    pub fn current_function(&self) -> Rc<RefCell<Function<'ctx>>> {
        self.current_function
            .clone()
            .expect("Must be declared before use")
    }

    ///
    /// Sets the current active function.
    ///
    pub fn set_current_function(&mut self, name: &str) -> anyhow::Result<()> {
        let function = self.functions.get(name).cloned().ok_or_else(|| {
            anyhow::anyhow!("Failed to activate an undeclared function `{}`", name)
        })?;
        self.current_function = Some(function);
        Ok(())
    }

    ///
    /// Pushes a new loop context to the stack.
    ///
    pub fn push_loop(
        &mut self,
        body_block: inkwell::basic_block::BasicBlock<'ctx>,
        continue_block: inkwell::basic_block::BasicBlock<'ctx>,
        join_block: inkwell::basic_block::BasicBlock<'ctx>,
    ) {
        self.loop_stack
            .push(Loop::new(body_block, continue_block, join_block));
    }

    ///
    /// Pops the current loop context from the stack.
    ///
    pub fn pop_loop(&mut self) {
        self.loop_stack.pop();
    }

    ///
    /// Returns the current loop context.
    ///
    pub fn r#loop(&self) -> &Loop<'ctx> {
        self.loop_stack
            .last()
            .expect("The current context is not in a loop")
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
    /// Appends a new basic block to the current function.
    ///
    pub fn append_basic_block(&self, name: &str) -> inkwell::basic_block::BasicBlock<'ctx> {
        self.llvm
            .append_basic_block(self.current_function().borrow().declaration().value, name)
    }

    ///
    /// Sets the current basic block.
    ///
    pub fn set_basic_block(&self, block: inkwell::basic_block::BasicBlock<'ctx>) {
        self.builder.position_at_end(block);
    }

    ///
    /// Returns the current basic block.
    ///
    pub fn basic_block(&self) -> inkwell::basic_block::BasicBlock<'ctx> {
        self.builder.get_insert_block().expect("Always exists")
    }

    ///
    /// Builds a stack allocation instruction.
    ///
    /// Sets the alignment to 256 bits.
    ///
    pub fn build_alloca<T: BasicType<'ctx> + Clone + Copy>(
        &self,
        r#type: T,
        name: &str,
    ) -> Pointer<'ctx> {
        let pointer = self.builder.build_alloca(r#type, name);
        self.basic_block()
            .get_last_instruction()
            .expect("Always exists")
            .set_alignment(era_compiler_common::BYTE_LENGTH_FIELD as u32)
            .expect("Alignment is valid");
        Pointer::new(r#type, AddressSpace::Stack, pointer)
    }

    ///
    /// Builds a stack load instruction.
    ///
    /// Sets the alignment to 256 bits for the stack and 1 bit for the heap, parent, and child.
    ///
    pub fn build_load(
        &self,
        pointer: Pointer<'ctx>,
        name: &str,
    ) -> inkwell::values::BasicValueEnum<'ctx> {
        let value = self.builder.build_load(pointer.r#type, pointer.value, name);

        let alignment = if AddressSpace::Stack == pointer.address_space {
            era_compiler_common::BYTE_LENGTH_FIELD
        } else {
            era_compiler_common::BYTE_LENGTH_BYTE
        };

        self.basic_block()
            .get_last_instruction()
            .expect("Always exists")
            .set_alignment(alignment as u32)
            .expect("Alignment is valid");
        value
    }

    ///
    /// Builds a stack store instruction.
    ///
    /// Sets the alignment to 256 bits for the stack and 1 bit for the heap, parent, and child.
    ///
    pub fn build_store<V>(&self, pointer: Pointer<'ctx>, value: V)
    where
        V: BasicValue<'ctx>,
    {
        let instruction = self.builder.build_store(pointer.value, value);

        let alignment = if AddressSpace::Stack == pointer.address_space {
            era_compiler_common::BYTE_LENGTH_FIELD
        } else {
            era_compiler_common::BYTE_LENGTH_BYTE
        };

        instruction
            .set_alignment(alignment as u32)
            .expect("Alignment is valid");
    }

    ///
    /// Builds a GEP instruction.
    ///
    pub fn build_gep<T>(
        &self,
        pointer: Pointer<'ctx>,
        indexes: &[inkwell::values::IntValue<'ctx>],
        element_type: T,
        name: &str,
    ) -> Pointer<'ctx>
    where
        T: BasicType<'ctx>,
    {
        let value = unsafe {
            self.builder
                .build_gep(pointer.r#type, pointer.value, indexes, name)
        };
        Pointer::new(element_type, pointer.address_space, value)
    }

    ///
    /// Builds a conditional branch.
    ///
    /// Checks if there are no other terminators in the block.
    ///
    pub fn build_conditional_branch(
        &self,
        comparison: inkwell::values::IntValue<'ctx>,
        then_block: inkwell::basic_block::BasicBlock<'ctx>,
        else_block: inkwell::basic_block::BasicBlock<'ctx>,
    ) {
        if self.basic_block().get_terminator().is_some() {
            return;
        }

        self.builder
            .build_conditional_branch(comparison, then_block, else_block);
    }

    ///
    /// Builds an unconditional branch.
    ///
    /// Checks if there are no other terminators in the block.
    ///
    pub fn build_unconditional_branch(
        &self,
        destination_block: inkwell::basic_block::BasicBlock<'ctx>,
    ) {
        if self.basic_block().get_terminator().is_some() {
            return;
        }

        self.builder.build_unconditional_branch(destination_block);
    }

    ///
    /// Builds a call.
    ///
    pub fn build_call(
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

    ///
    /// Builds a memory copy call.
    ///
    /// Sets the alignment to `1`, since all non-stack memory pages have such alignment.
    ///
    pub fn build_memcpy(
        &self,
        function: FunctionDeclaration<'ctx>,
        destination: Pointer<'ctx>,
        source: Pointer<'ctx>,
        size: inkwell::values::IntValue<'ctx>,
        name: &str,
    ) {
        let call_site_value = self.builder.build_indirect_call(
            function.r#type,
            function.value.as_global_value().as_pointer_value(),
            &[
                destination.value.as_basic_value_enum().into(),
                source.value.as_basic_value_enum().into(),
                size.as_basic_value_enum().into(),
                self.bool_type().const_zero().as_basic_value_enum().into(),
            ],
            name,
        );

        call_site_value.set_alignment_attribute(inkwell::attributes::AttributeLoc::Param(0), 1);
        call_site_value.set_alignment_attribute(inkwell::attributes::AttributeLoc::Param(1), 1);
    }

    ///
    /// Builds a return.
    ///
    /// Checks if there are no other terminators in the block.
    ///
    pub fn build_return(&self, value: Option<&dyn BasicValue<'ctx>>) {
        if self.basic_block().get_terminator().is_some() {
            return;
        }

        self.builder.build_return(value);
    }

    ///
    /// Builds an unreachable.
    ///
    /// Checks if there are no other terminators in the block.
    ///
    pub fn build_unreachable(&self) {
        if self.basic_block().get_terminator().is_some() {
            return;
        }

        self.builder.build_unreachable();
    }

    ///
    /// Returns a boolean type constant.
    ///
    pub fn bool_const(&self, value: bool) -> inkwell::values::IntValue<'ctx> {
        self.bool_type().const_int(u64::from(value), false)
    }

    ///
    /// Returns an integer type constant.
    ///
    pub fn integer_const(&self, bit_length: usize, value: u64) -> inkwell::values::IntValue<'ctx> {
        self.integer_type(bit_length).const_int(value, false)
    }

    ///
    /// Returns a 256-bit field type constant.
    ///
    pub fn field_const(&self, value: u64) -> inkwell::values::IntValue<'ctx> {
        self.field_type().const_int(value, false)
    }

    ///
    /// Returns a 256-bit field type undefined value.
    ///
    pub fn field_undef(&self) -> inkwell::values::IntValue<'ctx> {
        self.field_type().get_undef()
    }

    ///
    /// Returns a field type constant from a decimal string.
    ///
    pub fn field_const_str_dec(&self, value: &str) -> inkwell::values::IntValue<'ctx> {
        self.field_type()
            .const_int_from_string(value, inkwell::types::StringRadix::Decimal)
            .unwrap_or_else(|| panic!("Invalid string constant `{value}`"))
    }

    ///
    /// Returns a field type constant from a hexadecimal string.
    ///
    pub fn field_const_str_hex(&self, value: &str) -> inkwell::values::IntValue<'ctx> {
        self.field_type()
            .const_int_from_string(
                value.strip_prefix("0x").unwrap_or(value),
                inkwell::types::StringRadix::Hexadecimal,
            )
            .unwrap_or_else(|| panic!("Invalid string constant `{value}`"))
    }

    ///
    /// Returns the void type.
    ///
    pub fn void_type(&self) -> inkwell::types::VoidType<'ctx> {
        self.llvm.void_type()
    }

    ///
    /// Returns the boolean type.
    ///
    pub fn bool_type(&self) -> inkwell::types::IntType<'ctx> {
        self.llvm.bool_type()
    }

    ///
    /// Returns the default byte type.
    ///
    pub fn byte_type(&self) -> inkwell::types::IntType<'ctx> {
        self.llvm
            .custom_width_int_type(era_compiler_common::BIT_LENGTH_BYTE as u32)
    }

    ///
    /// Returns the integer type of the specified bit-length.
    ///
    pub fn integer_type(&self, bit_length: usize) -> inkwell::types::IntType<'ctx> {
        self.llvm.custom_width_int_type(bit_length as u32)
    }

    ///
    /// Returns the default field type.
    ///
    pub fn field_type(&self) -> inkwell::types::IntType<'ctx> {
        self.llvm
            .custom_width_int_type(era_compiler_common::BIT_LENGTH_FIELD as u32)
    }

    ///
    /// Returns the array type with the specified length.
    ///
    pub fn array_type<T>(&self, element_type: T, length: usize) -> inkwell::types::ArrayType<'ctx>
    where
        T: BasicType<'ctx>,
    {
        element_type.array_type(length as u32)
    }

    ///
    /// Returns the structure type with specified fields.
    ///
    pub fn structure_type<T>(&self, field_types: &[T]) -> inkwell::types::StructType<'ctx>
    where
        T: BasicType<'ctx>,
    {
        let field_types: Vec<inkwell::types::BasicTypeEnum<'ctx>> =
            field_types.iter().map(T::as_basic_type_enum).collect();
        self.llvm.struct_type(field_types.as_slice(), false)
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
            .expect("The EVMLA data must have been initialized")
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
            .expect("The EVMLA data must have been initialized")
    }
}
