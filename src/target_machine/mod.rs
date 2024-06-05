//!
//! The LLVM target machine.
//!

pub mod target;

use crate::optimizer::settings::size_level::SizeLevel as OptimizerSettingsSizeLevel;
use crate::optimizer::settings::Settings as OptimizerSettings;

use self::target::Target;

///
/// The LLVM target machine.
///
#[derive(Debug)]
pub struct TargetMachine {
    /// The LLVM target.
    target: Target,
    /// The LLVM target machine reference.
    target_machine: inkwell::targets::TargetMachine,
    /// The optimizer settings.
    optimizer_settings: OptimizerSettings,
}

impl TargetMachine {
    /// The LLVM target name.
    pub const VM_TARGET_NAME: &'static str = "eravm";

    /// The LLVM target triple.
    pub const VM_TARGET_TRIPLE: &'static str = "eravm-unknown-unknown";

    /// The actual production VM name.
    pub const VM_PRODUCTION_NAME: &'static str = "EraVM";

    ///
    /// A shortcut constructor.
    ///
    /// A separate instance for every optimization level is created.
    ///
    pub fn new(
        target: Target,
        optimizer_settings: &OptimizerSettings,
        llvm_options: &[String],
    ) -> anyhow::Result<Self> {
        let mut arguments = Vec::with_capacity(3 + llvm_options.len());
        arguments.push(target.name().to_owned());
        if optimizer_settings.is_system_request_memoization_disabled() {
            arguments.push("-eravm-disable-sha3-sreq-cse".to_owned());
        }
        if let Some(value) = optimizer_settings.jump_table_density_threshold() {
            arguments.push("-eravm-jump-table-density-threshold".to_owned());
            arguments.push(value.to_string());
        }
        arguments.extend_from_slice(llvm_options);
        if arguments.len() > 1 {
            let arguments: Vec<&str> = arguments.iter().map(|argument| argument.as_str()).collect();
            inkwell::support::parse_command_line_options(
                arguments.len() as i32,
                arguments.as_slice(),
                "Optimizer and extra parameters",
            );
        }

        let target_machine = inkwell::targets::Target::from_name(target.name())
            .ok_or_else(|| anyhow::anyhow!("LLVM target machine `{}` not found", target.name()))?
            .create_target_machine(
                &inkwell::targets::TargetTriple::create(target.triple()),
                "",
                "",
                optimizer_settings.level_back_end,
                inkwell::targets::RelocMode::Default,
                inkwell::targets::CodeModel::Default,
            )
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "LLVM target machine `{}` initialization error",
                    target.name(),
                )
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
    /// Writes the LLVM module to a memory buffer.
    ///
    pub fn write_to_memory_buffer(
        &self,
        module: &inkwell::module::Module,
    ) -> Result<inkwell::memory_buffer::MemoryBuffer, inkwell::support::LLVMString> {
        match self.target {
            Target::EraVM => self
                .target_machine
                .write_to_memory_buffer(module, inkwell::targets::FileType::Assembly),
            Target::EVM => self
                .target_machine
                .write_to_memory_buffer(module, inkwell::targets::FileType::Object),
        }
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

        if let Target::EraVM = self.target {
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
