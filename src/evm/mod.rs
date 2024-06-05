//!
//! The LLVM context library.
//!

pub mod r#const;
pub mod context;
pub mod instructions;

use crate::debug_config::DebugConfig;
use crate::optimizer::settings::Settings as OptimizerSettings;

use self::context::Context;

///
/// Initializes the EVM target machine.
///
pub fn initialize_target() {
    inkwell::targets::Target::initialize_evm(&inkwell::targets::InitializationConfig::default());
}

///
/// Implemented by items which are translated into LLVM IR.
///
#[allow(clippy::upper_case_acronyms)]
pub trait WriteLLVM<D>
where
    D: Dependency + Clone,
{
    ///
    /// Declares the entity in the LLVM IR.
    /// Is usually performed in order to use the item before defining it.
    ///
    fn declare(&mut self, _context: &mut Context<D>) -> anyhow::Result<()> {
        Ok(())
    }

    ///
    /// Translates the entity into LLVM IR.
    ///
    fn into_llvm(self, context: &mut Context<D>) -> anyhow::Result<()>;
}

///
/// The dummy LLVM writable entity.
///
#[derive(Debug, Default, Clone)]
pub struct DummyLLVMWritable {}

impl<D> WriteLLVM<D> for DummyLLVMWritable
where
    D: Dependency + Clone,
{
    fn into_llvm(self, _context: &mut Context<D>) -> anyhow::Result<()> {
        Ok(())
    }
}

///
/// Implemented by items managing project dependencies.
///
pub trait Dependency {
    ///
    /// Compiles a project dependency.
    ///
    fn compile(
        dependency: Self,
        path: &str,
        optimizer_settings: OptimizerSettings,
        llvm_options: &[String],
        include_metadata_hash: bool,
        debug_config: Option<DebugConfig>,
    ) -> anyhow::Result<String>;

    ///
    /// Resolves a full contract path.
    ///
    fn resolve_path(&self, identifier: &str) -> anyhow::Result<String>;

    ///
    /// Resolves a library address.
    ///
    fn resolve_library(&self, path: &str) -> anyhow::Result<String>;
}

///
/// The dummy dependency entity.
///
#[derive(Debug, Default, Clone)]
pub struct DummyDependency {}

impl Dependency for DummyDependency {
    fn compile(
        _dependency: Self,
        _path: &str,
        _optimizer_settings: OptimizerSettings,
        _llvm_options: &[String],
        _include_metadata_hash: bool,
        _debug_config: Option<DebugConfig>,
    ) -> anyhow::Result<String> {
        Ok(String::new())
    }

    ///
    /// Resolves a full contract path.
    ///
    fn resolve_path(&self, _identifier: &str) -> anyhow::Result<String> {
        Ok(String::new())
    }

    ///
    /// Resolves a library address.
    ///
    fn resolve_library(&self, _path: &str) -> anyhow::Result<String> {
        Ok(String::new())
    }
}
