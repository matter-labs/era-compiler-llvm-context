//!
//! The LLVM target machine.
//!

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use once_cell::sync::OnceCell;

use crate::context::optimizer::settings::Settings as OptimizerSettings;

///
/// The LLVM target machine.
///
/// The inner target machine reference is wrapped into a mutex, since inside of the LLVM framework
/// it is completely unsafe and causes data races in multi-threaded environments.
///
#[derive(Debug, Clone)]
pub struct TargetMachine {
    /// The inner LLVM target machine reference.
    target_machine: Arc<Mutex<inkwell::targets::TargetMachine>>,
}

unsafe impl Send for TargetMachine {}
unsafe impl Sync for TargetMachine {}

/// The array of singletons for every optimization level.
static TARGET_MACHINES: [OnceCell<TargetMachine>; 4] = [
    OnceCell::new(),
    OnceCell::new(),
    OnceCell::new(),
    OnceCell::new(),
];

/// The mutex to allow simultaneous access to only one target machine.
static TARGET_MACHINE_LOCK: Mutex<()> = Mutex::new(());

impl TargetMachine {
    /// The LLVM target name.
    pub const VM_TARGET_NAME: &'static str = "syncvm";

    /// The LLVM target triple.
    pub const VM_TARGET_TRIPLE: &'static str = "syncvm-unknown-unknown";

    /// The actual production VM name.
    pub const VM_PRODUCTION_NAME: &'static str = "zkEVM";

    ///
    /// A shortcut constructor.
    ///
    /// A separate instance for every optimization level is created.
    ///
    pub fn new(settings: &OptimizerSettings) -> anyhow::Result<Self> {
        TARGET_MACHINES[settings.level_back_end as usize]
            .get_or_try_init(|| {
                let target_machine = inkwell::targets::Target::from_name(Self::VM_TARGET_NAME)
                    .ok_or_else(|| {
                        anyhow::anyhow!("LLVM target machine `{}` not found", Self::VM_TARGET_NAME)
                    })?
                    .create_target_machine(
                        &inkwell::targets::TargetTriple::create(Self::VM_TARGET_TRIPLE),
                        "",
                        "",
                        settings.level_back_end,
                        inkwell::targets::RelocMode::Default,
                        inkwell::targets::CodeModel::Default,
                    )
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "LLVM target machine `{}` initialization error",
                            Self::VM_TARGET_NAME
                        )
                    })?;
                Ok(Self {
                    target_machine: Arc::new(Mutex::new(target_machine)),
                })
            })
            .cloned()
    }

    ///
    /// Sets the target-specific data in the module.
    ///
    pub fn set_target_data(&self, module: &inkwell::module::Module) {
        let _guard = TARGET_MACHINE_LOCK.lock().expect("Sync");
        module.set_triple(&self.lock().get_triple());
        module.set_data_layout(&self.lock().get_target_data().get_data_layout());
    }

    ///
    /// Writes the LLVM module to a memory buffer.
    ///
    pub fn write_to_memory_buffer(
        &self,
        module: &inkwell::module::Module,
    ) -> Result<inkwell::memory_buffer::MemoryBuffer, inkwell::support::LLVMString> {
        let _guard = TARGET_MACHINE_LOCK.lock().expect("Sync");
        self.lock()
            .write_to_memory_buffer(module, inkwell::targets::FileType::Assembly)
    }

    ///
    /// Runs the optimization passes on `module`.
    ///
    pub fn run_optimization_passes(
        &self,
        module: &inkwell::module::Module,
        passes: &str,
    ) -> Result<(), inkwell::support::LLVMString> {
        let _guard = TARGET_MACHINE_LOCK.lock().expect("Sync");
        module.run_passes(
            passes,
            &self.lock(),
            inkwell::passes::PassBuilderOptions::create(),
        )
    }

    ///
    /// Returns the target triple.
    ///
    pub fn get_triple(&self) -> inkwell::targets::TargetTriple {
        let _guard = TARGET_MACHINE_LOCK.lock().expect("Sync");
        self.lock().get_triple()
    }

    ///
    /// Returns the target data.
    ///
    pub fn get_target_data(&self) -> inkwell::targets::TargetData {
        let _guard = TARGET_MACHINE_LOCK.lock().expect("Sync");
        self.lock().get_target_data()
    }

    ///
    /// Returns the synchronized target machine reference.
    ///
    fn lock(&self) -> MutexGuard<inkwell::targets::TargetMachine> {
        self.target_machine.lock().expect("Sync")
    }
}
