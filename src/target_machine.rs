//!
//! The LLVM target machine.
//!

use crate::optimizer::settings::size_level::SizeLevel as OptimizerSettingsSizeLevel;
use crate::optimizer::settings::Settings as OptimizerSettings;

///
/// The LLVM target machine.
///
#[derive(Debug)]
pub struct TargetMachine {
    /// The LLVM target.
    target: era_compiler_common::Target,
    /// The LLVM target machine reference.
    target_machine: inkwell::targets::TargetMachine,
    /// The optimizer settings.
    optimizer_settings: OptimizerSettings,
}

impl TargetMachine {
    ///
    /// A shortcut constructor.
    ///
    /// Supported LLVM options for EraVM target:
    /// `-eravm-disable-sha3-sreq-cse`
    /// `-eravm-jump-table-density-threshold <value>`
    ///
    /// Supported LLVM options for EVM target:
    /// `-evm-stack-region-size <value>`
    /// `-evm-stack-region-offset <value>`
    ///
    pub fn new(
        target: era_compiler_common::Target,
        code_segment: Option<era_compiler_common::CodeSegment>,
        optimizer_settings: &OptimizerSettings,
        llvm_options: &[String],
    ) -> anyhow::Result<Self> {
        let mut arguments = Vec::with_capacity(1 + llvm_options.len());
        arguments.push(target.to_string());
        arguments.extend_from_slice(llvm_options);
        if let era_compiler_common::Target::EVM = target {
            let spill_area_size = match code_segment {
                Some(era_compiler_common::CodeSegment::Deploy) => {
                    optimizer_settings.deploy_code_spill_area_size
                }
                Some(era_compiler_common::CodeSegment::Runtime) => {
                    optimizer_settings.runtime_code_spill_area_size
                }
                None => None,
            };
            if let Some(size) = spill_area_size {
                arguments.push(format!(
                    "-evm-stack-region-offset={}",
                    crate::evm::r#const::SOLC_GENERAL_MEMORY_OFFSET
                ));
                arguments.push(format!("-evm-stack-region-size={size}"));
            }
        }
        if arguments.len() > 1 {
            let arguments: Vec<&str> = arguments.iter().map(|argument| argument.as_str()).collect();
            inkwell::support::parse_command_line_options(arguments.as_slice(), "LLVM options");
        }

        let target_machine = inkwell::targets::Target::from_name(target.to_string().as_str())
            .ok_or_else(|| anyhow::anyhow!("LLVM target machine `{target}` not found"))?
            .create_target_machine(
                &inkwell::targets::TargetTriple::create(target.triple()),
                "",
                "",
                optimizer_settings.level_back_end,
                inkwell::targets::RelocMode::Default,
                inkwell::targets::CodeModel::Default,
            )
            .ok_or_else(|| {
                anyhow::anyhow!("LLVM target machine `{target}` initialization error")
            })?;

        Ok(Self {
            target,
            target_machine,
            optimizer_settings: optimizer_settings.to_owned(),
        })
    }

    ///
    /// Sets the target-specific data in the module.
    ///
    pub fn set_target_data(&self, module: &inkwell::module::Module) {
        module.set_triple(&self.target_machine.get_triple());
        module.set_data_layout(&self.target_machine.get_target_data().get_data_layout());
    }

    ///
    /// Sets the assembly printer verbosity.
    ///
    pub fn set_asm_verbosity(&self, verbosity: bool) {
        self.target_machine.set_asm_verbosity(verbosity);
    }

    ///
    /// Translates textual assembly to the object code.
    ///
    pub fn assemble(
        &self,
        memory_buffer: &inkwell::memory_buffer::MemoryBuffer,
    ) -> Result<inkwell::memory_buffer::MemoryBuffer, inkwell::support::LLVMString> {
        memory_buffer.assemble_eravm(&self.target_machine)
    }

    ///
    /// Disassembles bytecode into textual representation.
    ///
    pub fn disassemble(
        &self,
        memory_buffer: &inkwell::memory_buffer::MemoryBuffer,
        pc: u64,
        options: u64,
    ) -> Result<inkwell::memory_buffer::MemoryBuffer, inkwell::support::LLVMString> {
        memory_buffer.disassemble_eravm(&self.target_machine, pc, options)
    }

    ///
    /// Writes the LLVM module to a memory buffer.
    ///
    pub fn write_to_memory_buffer(
        &self,
        module: &inkwell::module::Module,
        file_type: inkwell::targets::FileType,
    ) -> Result<inkwell::memory_buffer::MemoryBuffer, inkwell::support::LLVMString> {
        self.target_machine
            .write_to_memory_buffer(module, file_type)
    }

    ///
    /// Runs the optimization passes on `module`.
    ///
    pub fn run_optimization_passes(
        &self,
        module: &inkwell::module::Module,
        passes: &str,
    ) -> Result<(), inkwell::support::LLVMString> {
        let pass_builder_options = inkwell::passes::PassBuilderOptions::create();
        pass_builder_options.set_verify_each(self.optimizer_settings.is_verify_each_enabled);
        pass_builder_options.set_debug_logging(self.optimizer_settings.is_debug_logging_enabled);

        if let era_compiler_common::Target::EraVM = self.target {
            pass_builder_options.set_loop_unrolling(
                self.optimizer_settings.level_middle_end_size == OptimizerSettingsSizeLevel::Zero,
            );
            pass_builder_options.set_merge_functions(true);
        }

        module.run_passes(passes, &self.target_machine, pass_builder_options)
    }

    ///
    /// Returns the target triple.
    ///
    pub fn get_triple(&self) -> inkwell::targets::TargetTriple {
        self.target_machine.get_triple()
    }

    ///
    /// Returns the target data.
    ///
    pub fn get_target_data(&self) -> inkwell::targets::TargetData {
        self.target_machine.get_target_data()
    }
}
