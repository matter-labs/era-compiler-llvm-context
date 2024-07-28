//!
//! The LLVM IR generator context.
//!

pub mod address_space;
pub mod build;
pub mod evmla_data;
pub mod function;
pub mod global;
pub mod solidity_data;
pub mod vyper_data;
pub mod yul_data;

#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use inkwell::types::BasicType;
use inkwell::values::BasicValue;

use crate::context::attribute::Attribute;
use crate::context::code_type::CodeType;
use crate::context::function::declaration::Declaration as FunctionDeclaration;
use crate::context::function::r#return::Return as FunctionReturn;
use crate::context::pointer::Pointer;
use crate::context::r#loop::Loop;
use crate::context::IContext;
use crate::debug_info::DebugInfo;
use crate::eravm::DebugConfig;
use crate::eravm::Dependency;
use crate::optimizer::settings::Settings as OptimizerSettings;
use crate::optimizer::Optimizer;
use crate::target_machine::target::Target;
use crate::target_machine::TargetMachine;

use self::address_space::AddressSpace;
use self::build::Build;
use self::evmla_data::EVMLAData;
use self::function::intrinsics::Intrinsics;
use self::function::llvm_runtime::LLVMRuntime;
use self::function::Function;
use self::global::Global;
use self::solidity_data::SolidityData;
use self::vyper_data::VyperData;
use self::yul_data::YulData;

///
/// The LLVM IR generator context.
///
/// It is a not-so-big god-like object glueing all the compilers' complexity and act as an adapter
/// and a superstructure over the inner `inkwell` LLVM context.
///
pub struct Context<'ctx, D>
where
    D: Dependency,
{
    /// The inner LLVM context.
    llvm: &'ctx inkwell::context::Context,
    /// The inner LLVM context builder.
    builder: inkwell::builder::Builder<'ctx>,
    /// The optimization tools.
    optimizer: Optimizer,
    /// The current module.
    module: inkwell::module::Module<'ctx>,
    /// The extra LLVM options.
    llvm_options: Vec<String>,
    /// The current contract code type, which can be deploy or runtime.
    code_type: Option<CodeType>,
    /// The global variables.
    globals: HashMap<String, Global<'ctx>>,
    /// The LLVM intrinsic functions, defined on the LLVM side.
    intrinsics: Intrinsics<'ctx>,
    /// The LLVM runtime functions, defined on the LLVM side.
    llvm_runtime: LLVMRuntime<'ctx>,
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
    /// The debug info of the current module.
    debug_info: DebugInfo<'ctx>,
    /// The debug configuration telling whether to dump the needed IRs.
    debug_config: Option<DebugConfig>,

    /// The Solidity data.
    solidity_data: Option<SolidityData>,
    /// The Yul data.
    yul_data: Option<YulData>,
    /// The EVM legacy assembly data.
    evmla_data: Option<EVMLAData<'ctx>>,
    /// The Vyper data.
    vyper_data: Option<VyperData>,
}

impl<'ctx, D> Context<'ctx, D>
where
    D: Dependency,
{
    /// The functions hashmap default capacity.
    const FUNCTIONS_HASHMAP_INITIAL_CAPACITY: usize = 64;

    /// The globals hashmap default capacity.
    const GLOBALS_HASHMAP_INITIAL_CAPACITY: usize = 4;

    /// The loop stack default capacity.
    const LOOP_STACK_INITIAL_CAPACITY: usize = 16;

    ///
    /// Initializes a new LLVM context.
    ///
    pub fn new(
        llvm: &'ctx inkwell::context::Context,
        module: inkwell::module::Module<'ctx>,
        llvm_options: Vec<String>,
        optimizer: Optimizer,
        dependency_manager: Option<D>,
        debug_config: Option<DebugConfig>,
    ) -> Self {
        let builder = llvm.create_builder();
        let intrinsics = Intrinsics::new(llvm, &module);
        let llvm_runtime = LLVMRuntime::new(llvm, &module, &optimizer);
        let debug_info = DebugInfo::new(&module);

        Self {
            llvm,
            builder,
            llvm_options,
            optimizer,
            module,
            code_type: None,
            globals: HashMap::with_capacity(Self::GLOBALS_HASHMAP_INITIAL_CAPACITY),
            intrinsics,
            llvm_runtime,
            functions: HashMap::with_capacity(Self::FUNCTIONS_HASHMAP_INITIAL_CAPACITY),
            current_function: None,
            loop_stack: Vec::with_capacity(Self::LOOP_STACK_INITIAL_CAPACITY),

            dependency_manager,
            debug_info,
            debug_config,

            solidity_data: None,
            yul_data: None,
            evmla_data: None,
            vyper_data: None,
        }
    }

    ///
    /// Builds the LLVM IR module, returning the build artifacts.
    ///
    pub fn build(
        mut self,
        contract_path: &str,
        metadata_hash: Option<[u8; era_compiler_common::BYTE_LENGTH_FIELD]>,
        output_assembly: bool,
    ) -> anyhow::Result<Build> {
        let module_clone = self.module.clone();

        let target_machine = TargetMachine::new(
            Target::EraVM,
            self.optimizer.settings(),
            self.llvm_options.as_slice(),
        )?;
        target_machine.set_target_data(self.module());

        if let Some(ref debug_config) = self.debug_config {
            debug_config.dump_llvm_ir_unoptimized(contract_path, self.code_type, self.module())?;
        }
        self.verify()
            .map_err(|error| anyhow::anyhow!("unoptimized LLVM IR verification: {error}",))?;

        self.optimizer
            .run(&target_machine, self.module())
            .map_err(|error| anyhow::anyhow!("optimizing: {error}",))?;
        if let Some(ref debug_config) = self.debug_config {
            debug_config.dump_llvm_ir_optimized(contract_path, self.code_type, self.module())?;
        }
        self.verify()
            .map_err(|error| anyhow::anyhow!("optimized LLVM IR verification: {error}",))?;

        let assembly_buffer = if output_assembly {
            let assembly_buffer = target_machine
                .write_to_memory_buffer(self.module(), inkwell::targets::FileType::Assembly)
                .map_err(|error| anyhow::anyhow!("assembly emitting: {error}"))?;

            if let Some(ref debug_config) = self.debug_config {
                let assembly_text = String::from_utf8_lossy(assembly_buffer.as_slice());
                debug_config.dump_assembly(contract_path, None, assembly_text.as_ref())?;
            }

            Some(assembly_buffer)
        } else {
            None
        };

        let bytecode_buffer = match assembly_buffer {
            Some(ref assembly_buffer) => target_machine
                .assemble(assembly_buffer)
                .map_err(|error| anyhow::anyhow!("assembling: {error}")),
            None => target_machine
                .write_to_memory_buffer(self.module(), inkwell::targets::FileType::Object)
                .map_err(|error| anyhow::anyhow!("bytecode emitting: {error}")),
        }?;

        let metadata_size = metadata_hash
            .as_ref()
            .map(|array| array.len())
            .unwrap_or_default();

        if bytecode_buffer.exceeds_size_limit_eravm(metadata_size) {
            if self.optimizer.settings() != &OptimizerSettings::size()
                && self.optimizer.settings().is_fallback_to_size_enabled()
            {
                self.optimizer = Optimizer::new(OptimizerSettings::size());
                self.module = module_clone;
                return self
                    .build(contract_path, metadata_hash, output_assembly)
                    .map_err(|error| {
                        anyhow::anyhow!("falling back to optimizing for size: {error}")
                    });
            } else {
                anyhow::bail!(
                    "bytecode size exceeds the limit of {} instructions",
                    1 << 16
                );
            }
        }

        let assembly_text = assembly_buffer
            .map(|assembly_buffer| String::from_utf8_lossy(assembly_buffer.as_slice()).to_string());

        crate::eravm::build(bytecode_buffer, metadata_hash, assembly_text)
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
    /// Returns the pointer to a global variable.
    ///
    pub fn get_global(&self, name: &str) -> anyhow::Result<Global<'ctx>> {
        match self.globals.get(name) {
            Some(global) => Ok(*global),
            None => anyhow::bail!("global variable `{name}` is not declared"),
        }
    }

    ///
    /// Returns the value of a global variable.
    ///
    pub fn get_global_value(
        &self,
        name: &str,
    ) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
        let global = self.get_global(name)?;
        self.build_load(global.into(), name)
    }

    ///
    /// Sets the value to a global variable.
    ///
    pub fn set_global<T, V>(
        &mut self,
        name: &str,
        r#type: T,
        address_space: AddressSpace,
        value: V,
    ) -> anyhow::Result<()>
    where
        T: BasicType<'ctx> + Clone + Copy,
        V: BasicValue<'ctx> + Clone + Copy,
    {
        match self.globals.get(name) {
            Some(global) => {
                let global = *global;
                self.build_store(global.into(), value)?;
            }
            None => {
                let global = Global::new(self, r#type, address_space, value, name)?;
                self.globals.insert(name.to_owned(), global);
            }
        }
        Ok(())
    }

    ///
    /// Returns the active pointer at `index`.
    ///
    pub fn get_active_pointer(
        &self,
        index: inkwell::values::IntValue<'ctx>,
    ) -> anyhow::Result<inkwell::values::PointerValue<'ctx>> {
        let active_pointer_array_global = self
            .globals
            .get(crate::eravm_const::GLOBAL_ACTIVE_POINTER_ARRAY)
            .expect("Always exists")
            .to_owned();
        let active_pointer_pointer = self.build_gep(
            active_pointer_array_global.into(),
            &[self.field_const(0), index],
            self.ptr_type(AddressSpace::Generic.into()),
            "active_pointer_pointer",
        )?;
        let active_pointer = self.build_load(active_pointer_pointer, "active_pointer")?;
        Ok(active_pointer.into_pointer_value())
    }

    ///
    /// Sets the active pointer at `index`.
    ///
    pub fn set_active_pointer(
        &self,
        index: inkwell::values::IntValue<'ctx>,
        pointer: inkwell::values::PointerValue<'ctx>,
    ) -> anyhow::Result<()> {
        let active_pointer_array_global = self
            .globals
            .get(crate::eravm_const::GLOBAL_ACTIVE_POINTER_ARRAY)
            .expect("Always exists")
            .to_owned();
        let active_pointer_pointer = self.build_gep(
            active_pointer_array_global.into(),
            &[self.field_const(0), index],
            self.ptr_type(AddressSpace::Generic.into()),
            "active_pointer_pointer",
        )?;
        self.build_store(active_pointer_pointer, pointer)?;
        Ok(())
    }

    ///
    /// Returns the LLVM intrinsics collection reference.
    ///
    pub fn intrinsics(&self) -> &Intrinsics<'ctx> {
        &self.intrinsics
    }

    ///
    /// Returns the LLVM runtime function collection reference.
    ///
    pub fn llvm_runtime(&self) -> &LLVMRuntime<'ctx> {
        &self.llvm_runtime
    }

    ///
    /// Get the contract dependency data.
    ///
    pub fn get_dependency_data(&mut self, identifier: &str) -> anyhow::Result<String> {
        if let Some(vyper_data) = self.vyper_data.as_mut() {
            vyper_data.set_is_minimal_proxy_used();
        }
        self.dependency_manager
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("The dependency manager is unset"))
            .and_then(|manager| manager.get(identifier))
    }

    ///
    /// Gets a full contract_path from the dependency manager.
    ///
    pub fn resolve_path(&self, identifier: &str) -> anyhow::Result<String> {
        self.dependency_manager
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("The dependency manager is unset"))
            .and_then(|manager| {
                let full_path = manager.resolve_path(identifier)?;
                Ok(full_path)
            })
    }

    ///
    /// Gets a deployed library address from the dependency manager.
    ///
    pub fn resolve_library(
        &self,
        identifier: &str,
    ) -> anyhow::Result<inkwell::values::IntValue<'ctx>> {
        self.dependency_manager
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("The dependency manager is unset"))
            .and_then(|manager| {
                let address = manager.resolve_library(identifier)?;
                let address = self.field_const_str_hex(address.as_str());
                Ok(address)
            })
    }

    ///
    /// Builds an invoke of local call covered with an exception handler.
    ///
    /// Yul does not the exception handling, so the user can declare a special handling function
    /// called (see constant `ZKSYNC_NEAR_CALL_ABI_EXCEPTION_HANDLER`. If the enclosed function
    /// panics, the control flow will be transferred to the exception handler.
    ///
    pub fn build_invoke_near_call_abi(
        &self,
        function: FunctionDeclaration<'ctx>,
        arguments: Vec<inkwell::values::BasicValueEnum<'ctx>>,
        name: &str,
    ) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
        let join_block = self.append_basic_block("near_call_join_block");

        let return_pointer = if let Some(r#type) = function.r#type.get_return_type() {
            let pointer = self.build_alloca(r#type, "near_call_return_pointer")?;
            self.build_store(pointer, r#type.const_zero())?;
            Some(pointer)
        } else {
            None
        };

        let call_site_value = if let Some(handler) = self
            .functions
            .get(Function::ZKSYNC_NEAR_CALL_ABI_EXCEPTION_HANDLER)
        {
            let success_block = self.append_basic_block("near_call_success_block");
            let catch_block = self.append_basic_block("near_call_catch_block");
            let current_block = self.basic_block();

            self.set_basic_block(catch_block);
            let landing_pad_type = self.structure_type(&[
                self.ptr_type(AddressSpace::Stack.into())
                    .as_basic_type_enum(),
                self.integer_type(era_compiler_common::BIT_LENGTH_X32)
                    .as_basic_type_enum(),
            ]);
            self.builder.build_landing_pad(
                landing_pad_type,
                self.llvm_runtime.personality.value,
                &[self
                    .ptr_type(AddressSpace::Stack.into())
                    .const_zero()
                    .as_basic_value_enum()],
                false,
                "near_call_catch_landing",
            )?;
            self.build_call(handler.borrow().declaration(), &[], "near_call_catch_call")?;
            self.build_unconditional_branch(join_block)?;

            self.set_basic_block(current_block);
            let call_site_value = self.builder.build_indirect_invoke(
                self.intrinsics.near_call.r#type,
                self.intrinsics
                    .near_call
                    .value
                    .as_global_value()
                    .as_pointer_value(),
                arguments.as_slice(),
                success_block,
                catch_block,
                name,
            )?;
            self.modify_call_site_value(
                arguments.as_slice(),
                call_site_value,
                self.intrinsics.near_call,
            );
            self.set_basic_block(success_block);
            call_site_value.try_as_basic_value().left()
        } else {
            self.build_call(self.intrinsics.near_call, arguments.as_slice(), name)?
        };

        if let (Some(return_pointer), Some(mut return_value)) = (return_pointer, call_site_value) {
            if let Some(return_type) = function.r#type.get_return_type() {
                if return_type.is_pointer_type() {
                    return_value = self
                        .builder()
                        .build_int_to_ptr(
                            return_value.into_int_value(),
                            return_type.into_pointer_type(),
                            format!("{name}_near_call_return_pointer_casted").as_str(),
                        )?
                        .as_basic_value_enum();
                }
            }
            self.build_store(return_pointer, return_value)?;
        }
        self.build_unconditional_branch(join_block)?;

        self.set_basic_block(join_block);
        match return_pointer {
            Some(pointer) => self.build_load(pointer, "near_call_result").map(Some),
            None => Ok(None),
        }
    }

    ///
    /// Builds a memory copy call for the return data.
    ///
    /// Sets the output length to `min(output_length, return_data_size` and calls the default
    /// generic page memory copy builder.
    ///
    pub fn build_memcpy_return_data(
        &self,
        function: FunctionDeclaration<'ctx>,
        destination: Pointer<'ctx, AddressSpace>,
        source: Pointer<'ctx, AddressSpace>,
        size: inkwell::values::IntValue<'ctx>,
        name: &str,
    ) -> anyhow::Result<()> {
        let pointer_casted = self.builder.build_ptr_to_int(
            source.value,
            self.field_type(),
            format!("{name}_pointer_casted").as_str(),
        )?;
        let return_data_size_shifted = self.builder.build_right_shift(
            pointer_casted,
            self.field_const((era_compiler_common::BIT_LENGTH_X32 * 3) as u64),
            false,
            format!("{name}_return_data_size_shifted").as_str(),
        )?;
        let return_data_size_truncated = self.builder.build_and(
            return_data_size_shifted,
            self.field_const(u32::MAX as u64),
            format!("{name}_return_data_size_truncated").as_str(),
        )?;
        let is_return_data_size_lesser = self.builder.build_int_compare(
            inkwell::IntPredicate::ULT,
            return_data_size_truncated,
            size,
            format!("{name}_is_return_data_size_lesser").as_str(),
        )?;
        let min_size = self
            .builder
            .build_select(
                is_return_data_size_lesser,
                return_data_size_truncated,
                size,
                format!("{name}_min_size").as_str(),
            )?
            .into_int_value();

        self.build_memcpy(function, destination, source, min_size, name)
    }

    ///
    /// Builds a long contract exit sequence.
    ///
    /// The deploy code does not return the runtime code like in EVM. Instead, it returns some
    /// additional contract metadata, e.g. the array of immutables.
    /// The deploy code uses the auxiliary heap for the return, because otherwise it is not possible
    /// to allocate memory together with the Yul allocator safely.
    ///
    pub fn build_exit(
        &self,
        return_function: FunctionDeclaration<'ctx>,
        offset: inkwell::values::IntValue<'ctx>,
        length: inkwell::values::IntValue<'ctx>,
    ) -> anyhow::Result<()> {
        let return_forward_mode = if self.code_type() == Some(CodeType::Deploy)
            && return_function == self.llvm_runtime().r#return
        {
            zkevm_opcode_defs::RetForwardPageType::UseAuxHeap
        } else {
            zkevm_opcode_defs::RetForwardPageType::UseHeap
        };

        self.build_call(
            return_function,
            &[
                offset.as_basic_value_enum(),
                length.as_basic_value_enum(),
                self.field_const(return_forward_mode as u64)
                    .as_basic_value_enum(),
            ],
            "exit_call",
        )?;
        self.builder.build_unreachable()?;
        Ok(())
    }

    ///
    /// Writes the ABI pointer to the global variable.
    ///
    pub fn write_abi_pointer(
        &mut self,
        pointer: Pointer<'ctx, AddressSpace>,
        global_name: &str,
    ) -> anyhow::Result<()> {
        self.set_global(
            global_name,
            self.ptr_type(AddressSpace::Generic.into()),
            AddressSpace::Stack,
            pointer.value,
        )
    }

    ///
    /// Writes the ABI data size to the global variable.
    ///
    pub fn write_abi_data_size(
        &mut self,
        pointer: Pointer<'ctx, AddressSpace>,
        global_name: &str,
    ) -> anyhow::Result<()> {
        let abi_pointer_value = self.builder().build_ptr_to_int(
            pointer.value,
            self.field_type(),
            "abi_pointer_value",
        )?;
        let abi_pointer_value_shifted = self.builder().build_right_shift(
            abi_pointer_value,
            self.field_const((era_compiler_common::BIT_LENGTH_X32 * 3) as u64),
            false,
            "abi_pointer_value_shifted",
        )?;
        let abi_length_value = self.builder().build_and(
            abi_pointer_value_shifted,
            self.field_const(u32::MAX as u64),
            "abi_length_value",
        )?;
        self.set_global(
            global_name,
            self.field_type(),
            AddressSpace::Stack,
            abi_length_value,
        )?;
        Ok(())
    }

    ///
    /// Returns a Yul function type with the specified arguments and number of return values.
    ///
    pub fn function_type<T>(
        &self,
        argument_types: Vec<T>,
        return_values_size: usize,
        is_near_call_abi: bool,
    ) -> inkwell::types::FunctionType<'ctx>
    where
        T: BasicType<'ctx>,
    {
        let mut argument_types: Vec<inkwell::types::BasicMetadataTypeEnum> = argument_types
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
            _size if is_near_call_abi && self.are_eravm_extensions_enabled() => {
                let return_type = self.ptr_type(AddressSpace::Stack.into());
                argument_types.insert(0, return_type.as_basic_type_enum().into());
                return_type.fn_type(argument_types.as_slice(), false)
            }
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
                if function == self.llvm_runtime().mstore8 {
                    call_site_value.add_attribute(
                        inkwell::attributes::AttributeLoc::Param(index as u32),
                        self.llvm.create_string_attribute("memory", "write"),
                    );
                }
                if function == self.llvm_runtime().sha3 {
                    call_site_value.add_attribute(
                        inkwell::attributes::AttributeLoc::Param(index as u32),
                        self.llvm.create_string_attribute("memory", "read"),
                    );
                }
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
    /// Returns the current number of immutables values in the contract.
    ///
    /// # Panics
    /// If the value is not set is any of the data sources.
    ///
    pub fn immutables_size(&self) -> usize {
        if let Some(solidity) = self.solidity_data.as_ref() {
            solidity.immutables_size()
        } else if let Some(vyper) = self.vyper_data.as_ref() {
            vyper.immutables_size()
        } else {
            panic!("The immutable size data is not available");
        }
    }

    ///
    /// Whether the EraVM extensions are enabled.
    ///
    pub fn are_eravm_extensions_enabled(&self) -> bool {
        self.yul_data
            .as_ref()
            .map(|data| data.are_eravm_extensions_enabled())
            .unwrap_or_default()
    }
}

impl<'ctx, D> IContext<'ctx> for Context<'ctx, D>
where
    D: Dependency,
{
    type Function = Function<'ctx>;

    type AddressSpace = AddressSpace;

    type SolidityData = SolidityData;

    type YulData = YulData;

    type EVMLAData = EVMLAData<'ctx>;

    type VyperData = VyperData;

    fn llvm(&self) -> &'ctx inkwell::context::Context {
        self.llvm
    }

    fn builder(&self) -> &inkwell::builder::Builder<'ctx> {
        &self.builder
    }

    fn module(&self) -> &inkwell::module::Module<'ctx> {
        &self.module
    }

    fn debug_info(&self) -> &DebugInfo<'ctx> {
        &self.debug_info
    }

    fn debug_config(&self) -> Option<&DebugConfig> {
        self.debug_config.as_ref()
    }

    fn set_code_type(&mut self, code_type: CodeType) {
        self.code_type = Some(code_type);
    }

    fn code_type(&self) -> Option<CodeType> {
        self.code_type.to_owned()
    }

    fn append_basic_block(&self, name: &str) -> inkwell::basic_block::BasicBlock<'ctx> {
        self.llvm
            .append_basic_block(self.current_function().borrow().declaration().value, name)
    }

    fn set_basic_block(&self, block: inkwell::basic_block::BasicBlock<'ctx>) {
        self.builder.position_at_end(block);
    }

    fn basic_block(&self) -> inkwell::basic_block::BasicBlock<'ctx> {
        self.builder.get_insert_block().expect("Always exists")
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
        mut linkage: Option<inkwell::module::Linkage>,
    ) -> anyhow::Result<Rc<RefCell<Function<'ctx>>>> {
        if Function::is_near_call_abi(name) && self.are_eravm_extensions_enabled() {
            linkage = Some(inkwell::module::Linkage::External);
        }

        let value = self.module().add_function(name, r#type, linkage);

        let entry_block = self.llvm.append_basic_block(value, "entry");
        let return_block = self.llvm.append_basic_block(value, "return");

        value.set_personality_function(self.llvm_runtime.personality.value);

        let r#return = match return_values_length {
            0 => FunctionReturn::none(),
            1 => {
                self.set_basic_block(entry_block);
                let pointer = self.build_alloca(self.field_type(), "return_pointer")?;
                FunctionReturn::primitive(pointer)
            }
            size if name.starts_with(Function::ZKSYNC_NEAR_CALL_ABI_PREFIX) => {
                let first_argument = value.get_first_param().expect("Always exists");
                let r#type = self.structure_type(vec![self.field_type(); size].as_slice());
                let pointer = first_argument.into_pointer_value();
                FunctionReturn::compound(Pointer::new(r#type, AddressSpace::Stack, pointer), size)
            }
            size => {
                self.set_basic_block(entry_block);
                let pointer = self.build_alloca(
                    self.structure_type(
                        vec![self.field_type().as_basic_type_enum(); size].as_slice(),
                    ),
                    "return_pointer",
                )?;
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
        if Function::is_near_call_abi(function.name()) && self.are_eravm_extensions_enabled() {
            Function::set_exception_handler_attributes(self.llvm, function.declaration());
        }

        let function = Rc::new(RefCell::new(function));
        self.functions.insert(name.to_string(), function.clone());

        Ok(function)
    }

    fn get_function(&self, name: &str) -> Option<Rc<RefCell<Function<'ctx>>>> {
        self.functions.get(name).cloned()
    }

    fn current_function(&self) -> Rc<RefCell<Function<'ctx>>> {
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
    ) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
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
        )?;
        self.modify_call_site_value(arguments, call_site_value, function);
        Ok(call_site_value.try_as_basic_value().left())
    }

    fn build_invoke(
        &self,
        function: FunctionDeclaration<'ctx>,
        arguments: &[inkwell::values::BasicValueEnum<'ctx>],
        name: &str,
    ) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
        if !self
            .functions
            .contains_key(Function::ZKSYNC_NEAR_CALL_ABI_EXCEPTION_HANDLER)
        {
            return self.build_call(function, arguments, name);
        }

        let return_pointer = if let Some(r#type) = function.r#type.get_return_type() {
            let pointer = self.build_alloca(r#type, "invoke_return_pointer")?;
            self.build_store(pointer, r#type.const_zero())?;
            Some(pointer)
        } else {
            None
        };

        let success_block = self.append_basic_block("invoke_success_block");
        let catch_block = self.append_basic_block("invoke_catch_block");
        let current_block = self.basic_block();

        self.set_basic_block(catch_block);
        let landing_pad_type = self.structure_type(&[
            self.ptr_type(AddressSpace::Stack.into())
                .as_basic_type_enum(),
            self.integer_type(era_compiler_common::BIT_LENGTH_X32)
                .as_basic_type_enum(),
        ]);
        self.builder.build_landing_pad(
            landing_pad_type,
            self.llvm_runtime.personality.value,
            &[self
                .ptr_type(AddressSpace::Stack.into())
                .const_zero()
                .as_basic_value_enum()],
            false,
            "invoke_catch_landing",
        )?;
        crate::eravm::utils::throw(self)?;

        self.set_basic_block(current_block);
        let call_site_value = self.builder.build_indirect_invoke(
            function.r#type,
            function.value.as_global_value().as_pointer_value(),
            arguments,
            success_block,
            catch_block,
            name,
        )?;
        self.modify_call_site_value(arguments, call_site_value, function);

        self.set_basic_block(success_block);
        if let (Some(return_pointer), Some(mut return_value)) =
            (return_pointer, call_site_value.try_as_basic_value().left())
        {
            if let Some(return_type) = function.r#type.get_return_type() {
                if return_type.is_pointer_type() {
                    return_value = self
                        .builder()
                        .build_int_to_ptr(
                            return_value.into_int_value(),
                            return_type.into_pointer_type(),
                            format!("{name}_invoke_return_pointer_casted").as_str(),
                        )?
                        .as_basic_value_enum();
                }
            }
            self.build_store(return_pointer, return_value)?;
        }
        match return_pointer {
            Some(pointer) => self.build_load(pointer, "invoke_result").map(Some),
            None => Ok(None),
        }
    }

    fn set_solidity_data(&mut self, data: Self::SolidityData) {
        self.solidity_data = Some(data);
    }

    fn solidity(&self) -> &Self::SolidityData {
        self.solidity_data
            .as_ref()
            .expect("The Solidity data must have been initialized")
    }

    fn solidity_mut(&mut self) -> &mut Self::SolidityData {
        self.solidity_data
            .as_mut()
            .expect("The Solidity data must have been initialized")
    }

    fn set_yul_data(&mut self, data: Self::YulData) {
        self.yul_data = Some(data);
    }

    fn yul(&self) -> &Self::YulData {
        self.yul_data
            .as_ref()
            .expect("The Yul data must have been initialized")
    }

    fn yul_mut(&mut self) -> &mut Self::YulData {
        self.yul_data
            .as_mut()
            .expect("The Yul data must have been initialized")
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

    fn set_vyper_data(&mut self, data: Self::VyperData) {
        self.vyper_data = Some(data);
    }

    fn vyper(&self) -> &Self::VyperData {
        self.vyper_data
            .as_ref()
            .expect("The Vyper data must have been initialized")
    }
}
