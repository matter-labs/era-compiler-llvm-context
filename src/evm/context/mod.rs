//!
//! The LLVM IR generator context.
//!

pub mod address_space;
pub mod evmla_data;
pub mod function;
pub mod solidity_data;
pub mod yul_data;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use inkwell::types::BasicType;

use crate::context::attribute::Attribute;
use crate::context::function::declaration::Declaration as FunctionDeclaration;
use crate::context::function::r#return::Return as FunctionReturn;
use crate::context::r#loop::Loop;
use crate::context::IContext;
use crate::debug_config::DebugConfig;
use crate::evm::build::Build as EVMBuild;
use crate::evm::profiler::Profiler;
use crate::evm::warning::Warning;
use crate::optimizer::settings::Settings as OptimizerSettings;
use crate::optimizer::Optimizer;
use crate::target_machine::TargetMachine;

use self::address_space::AddressSpace;
use self::evmla_data::EVMLAData;
use self::function::intrinsics::Intrinsics;
use self::function::Function;
use self::solidity_data::SolidityData;
use self::yul_data::YulData;

///
/// The LLVM IR generator context.
///
/// It is a not-so-big god-like object glueing all the compilers' complexity and act as an adapter
/// and a superstructure over the inner `inkwell` LLVM context.
///
pub struct Context<'ctx> {
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
    code_segment: era_compiler_common::CodeSegment,
    /// The LLVM intrinsic functions, defined on the LLVM side.
    intrinsics: Intrinsics<'ctx>,
    /// The declared functions.
    functions: HashMap<String, Rc<RefCell<Function<'ctx>>>>,
    /// The current active function.
    current_function: Option<Rc<RefCell<Function<'ctx>>>>,
    /// The loop context stack.
    loop_stack: Vec<Loop<'ctx>>,

    /// The debug configuration telling whether to dump the needed IRs.
    debug_config: Option<DebugConfig>,

    /// The Solidity data.
    solidity_data: Option<SolidityData>,
    /// The Yul data.
    yul_data: Option<YulData>,
    /// The EVM legacy assembly data.
    evmla_data: Option<EVMLAData<'ctx>>,
}

impl<'ctx> Context<'ctx> {
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
        llvm_options: Vec<String>,
        code_segment: era_compiler_common::CodeSegment,
        optimizer: Optimizer,
        debug_config: Option<DebugConfig>,
    ) -> Self {
        let builder = llvm.create_builder();
        let intrinsics = Intrinsics::new(llvm, &module);

        Self {
            llvm,
            builder,
            llvm_options,
            optimizer,
            module,
            code_segment,
            intrinsics,
            functions: HashMap::with_capacity(Self::FUNCTIONS_HASHMAP_INITIAL_CAPACITY),
            current_function: None,
            loop_stack: Vec::with_capacity(Self::LOOP_STACK_INITIAL_CAPACITY),

            debug_config,

            solidity_data: None,
            yul_data: None,
            evmla_data: None,
        }
    }

    ///
    /// Builds the LLVM IR module, returning the build artifacts.
    ///
    pub fn build(
        &mut self,
        output_assembly: bool,
        output_bytecode: bool,
        is_size_fallback: bool,
        profiler: &mut Profiler,
    ) -> anyhow::Result<EVMBuild> {
        let module_clone = self.module.clone();
        let contract_path = self.module.get_name().to_str().expect("Always valid");

        let run_init_verify = profiler.start_evm_translation_unit(
            contract_path,
            self.code_segment,
            "InitVerify",
            self.optimizer.settings(),
        );
        let target_machine = TargetMachine::new(
            era_compiler_common::Target::EVM,
            self.optimizer.settings(),
            self.llvm_options.as_slice(),
        )?;
        target_machine.set_target_data(self.module());
        target_machine.set_asm_verbosity(true);

        let spill_area = self
            .optimizer
            .settings()
            .spill_area_size()
            .map(|spill_area_size| {
                (
                    crate::evm::r#const::SOLC_GENERAL_MEMORY_OFFSET,
                    spill_area_size,
                )
            });

        if let Some(ref debug_config) = self.debug_config {
            debug_config.dump_llvm_ir_unoptimized(
                contract_path,
                self.module(),
                is_size_fallback,
                spill_area,
            )?;
        }
        self.verify().map_err(|error| {
            anyhow::anyhow!(
                "{} code unoptimized LLVM IR verification: {error}",
                self.code_segment,
            )
        })?;
        run_init_verify.borrow_mut().finish();

        let run_optimize_verify = profiler.start_evm_translation_unit(
            contract_path,
            self.code_segment,
            "OptimizeVerify",
            self.optimizer.settings(),
        );
        self.optimizer
            .run(&target_machine, self.module())
            .map_err(|error| anyhow::anyhow!("{} code optimizing: {error}", self.code_segment))?;
        if let Some(ref debug_config) = self.debug_config {
            debug_config.dump_llvm_ir_optimized(
                contract_path,
                self.module(),
                is_size_fallback,
                spill_area,
            )?;
        }
        self.verify().map_err(|error| {
            anyhow::anyhow!(
                "{} code optimized LLVM IR verification: {error}",
                self.code_segment,
            )
        })?;
        run_optimize_verify.borrow_mut().finish();

        let assembly_buffer = if output_assembly || self.debug_config.is_some() {
            let run_emit_llvm_assembly = profiler.start_evm_translation_unit(
                contract_path,
                self.code_segment,
                "EmitLLVMAssembly",
                self.optimizer.settings(),
            );
            let assembly_buffer = target_machine
                .write_to_memory_buffer(self.module(), inkwell::targets::FileType::Assembly)
                .map_err(|error| anyhow::anyhow!("assembly emitting: {error}"))?;

            if let Some(ref debug_config) = self.debug_config {
                let assembly_text = String::from_utf8_lossy(assembly_buffer.as_slice());
                debug_config.dump_assembly(
                    contract_path,
                    era_compiler_common::Target::EVM,
                    assembly_text.as_ref(),
                    is_size_fallback,
                    spill_area,
                )?;
            }

            run_emit_llvm_assembly.borrow_mut().finish();
            Some(assembly_buffer)
        } else {
            None
        };
        let assembly = assembly_buffer
            .map(|assembly_buffer| String::from_utf8_lossy(assembly_buffer.as_slice()).to_string());

        if output_bytecode {
            let run_emit_bytecode = profiler.start_evm_translation_unit(
                contract_path,
                self.code_segment,
                "EmitBytecode",
                self.optimizer.settings(),
            );
            let bytecode_buffer = target_machine
                .write_to_memory_buffer(self.module(), inkwell::targets::FileType::Object)
                .map_err(|error| {
                    anyhow::anyhow!("{} bytecode emitting: {error}", self.code_segment)
                })?;
            run_emit_bytecode.borrow_mut().finish();

            let immutables = match self.code_segment {
                era_compiler_common::CodeSegment::Deploy => None,
                era_compiler_common::CodeSegment::Runtime => {
                    Some(bytecode_buffer.get_immutables_evm())
                }
            };

            let bytecode_size_limit = match self.code_segment {
                era_compiler_common::CodeSegment::Deploy => {
                    crate::evm::r#const::DEPLOY_CODE_SIZE_LIMIT
                }
                era_compiler_common::CodeSegment::Runtime => {
                    crate::evm::r#const::RUNTIME_CODE_SIZE_LIMIT
                }
            };

            let mut warnings = Vec::with_capacity(1);
            let bytecode_size = bytecode_buffer.as_slice().len();
            if bytecode_size > bytecode_size_limit {
                if self.optimizer.settings() == &OptimizerSettings::cycles()
                    && self.optimizer.settings().is_fallback_to_size_enabled()
                {
                    crate::evm::IS_SIZE_FALLBACK
                        .compare_exchange(
                            false,
                            true,
                            std::sync::atomic::Ordering::Relaxed,
                            std::sync::atomic::Ordering::Relaxed,
                        )
                        .expect("Failed to set the global size fallback flag");
                    self.optimizer = Optimizer::new(OptimizerSettings::size());
                    self.module = module_clone;
                    for function in self.module.get_functions() {
                        Function::set_size_attributes(self.llvm, function);
                    }
                    return self.build(output_assembly, output_bytecode, true, profiler);
                } else {
                    warnings.push(match self.code_segment {
                        era_compiler_common::CodeSegment::Deploy => Warning::DeployCodeSize {
                            found: bytecode_size,
                        },
                        era_compiler_common::CodeSegment::Runtime => Warning::RuntimeCodeSize {
                            found: bytecode_size,
                        },
                    })
                };
            }
            Ok(EVMBuild::new(
                Some(bytecode_buffer.as_slice().to_vec()),
                assembly,
                immutables,
                is_size_fallback,
                warnings,
            ))
        } else {
            Ok(EVMBuild::new(
                None,
                assembly,
                None,
                is_size_fallback,
                vec![],
            ))
        }
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
        arguments: &[inkwell::values::BasicMetadataValueEnum<'ctx>],
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
                if (*argument)
                    .try_into()
                    .map(|argument: inkwell::values::BasicValueEnum<'ctx>| argument.get_type())
                    .ok()
                    == function.r#type.get_return_type()
                {
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

impl<'ctx> IContext<'ctx> for Context<'ctx> {
    type Function = Function<'ctx>;

    type AddressSpace = AddressSpace;

    type SolidityData = SolidityData;

    type YulData = YulData;

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

    fn optimizer(&self) -> &Optimizer {
        &self.optimizer
    }

    fn debug_config(&self) -> Option<&DebugConfig> {
        self.debug_config.as_ref()
    }

    fn set_code_segment(&mut self, code_segment: era_compiler_common::CodeSegment) {
        self.code_segment = code_segment;
    }

    fn code_segment(&self) -> Option<era_compiler_common::CodeSegment> {
        Some(self.code_segment.to_owned())
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
                let pointer = self.build_alloca(self.field_type(), "return_pointer")?;
                FunctionReturn::primitive(pointer)
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
    ) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
        let arguments: Vec<inkwell::values::BasicMetadataValueEnum> = arguments
            .iter()
            .copied()
            .map(inkwell::values::BasicMetadataValueEnum::from)
            .collect();
        self.build_call_metadata(function, arguments.as_slice(), name)
    }

    fn build_call_metadata(
        &self,
        function: FunctionDeclaration<'ctx>,
        arguments: &[inkwell::values::BasicMetadataValueEnum<'ctx>],
        name: &str,
    ) -> anyhow::Result<Option<inkwell::values::BasicValueEnum<'ctx>>> {
        let call_site_value = self.builder.build_indirect_call(
            function.r#type,
            function.value.as_global_value().as_pointer_value(),
            arguments,
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
        Self::build_call(self, function, arguments, name)
    }

    fn set_solidity_data(&mut self, data: Self::SolidityData) {
        self.solidity_data = Some(data);
    }

    fn solidity(&self) -> Option<&Self::SolidityData> {
        self.solidity_data.as_ref()
    }

    fn solidity_mut(&mut self) -> Option<&mut Self::SolidityData> {
        self.solidity_data.as_mut()
    }

    fn set_yul_data(&mut self, data: Self::YulData) {
        self.yul_data = Some(data);
    }

    fn yul(&self) -> Option<&Self::YulData> {
        self.yul_data.as_ref()
    }

    fn yul_mut(&mut self) -> Option<&mut Self::YulData> {
        self.yul_data.as_mut()
    }

    fn set_evmla_data(&mut self, data: Self::EVMLAData) {
        self.evmla_data = Some(data);
    }

    fn evmla(&self) -> Option<&Self::EVMLAData> {
        self.evmla_data.as_ref()
    }

    fn evmla_mut(&mut self) -> Option<&mut Self::EVMLAData> {
        self.evmla_data.as_mut()
    }

    fn set_vyper_data(&mut self, _data: Self::VyperData) {
        panic!("Unused with the EVM target");
    }

    fn vyper(&self) -> Option<&Self::VyperData> {
        panic!("Unused with the EVM target");
    }

    fn vyper_mut(&mut self) -> Option<&mut Self::VyperData> {
        panic!("Unused with the EVM target");
    }
}
