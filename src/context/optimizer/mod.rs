//!
//! The LLVM optimizing tools.
//!

pub mod settings;

use crate::context::target_machine::TargetMachine;

use self::settings::Settings;

///
/// The LLVM optimizing tools.
///
#[derive(Debug)]
pub struct Optimizer {
    /// The LLVM target machine.
    target_machine: TargetMachine,
    /// The optimizer settings.
    settings: Settings,
}

impl Optimizer {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(target_machine: TargetMachine, settings: Settings) -> Self {
        Self {
            target_machine,
            settings,
        }
    }

    ///
    /// Runs the new pass manager.
    ///
    pub fn run(
        &self,
        module: &inkwell::module::Module,
    ) -> Result<(), inkwell::support::LLVMString> {
        self.target_machine.run_optimization_passes(
            module,
            format!("default<O{}>", self.settings.middle_end_as_string()).as_str(),
        )
    }

    ///
    /// Returns the target machine reference.
    ///
    pub fn target_machine(&self) -> &TargetMachine {
        &self.target_machine
    }

    ///
    /// Returns the optimizer settings reference.
    ///
    pub fn settings(&self) -> &Settings {
        &self.settings
    }
}
