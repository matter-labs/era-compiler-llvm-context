//!
//! The LLVM context library.
//!

pub mod r#const;
pub mod context;
pub mod evm;
pub mod extensions;
pub mod metadata_hash;
pub mod utils;

pub use self::r#const::*;

use crate::debug_config::DebugConfig;
use crate::dependency::Dependency;

use self::context::Context;

///
/// Initializes the EraVM target machine.
///
pub fn initialize_target() {
    inkwell::targets::Target::initialize_eravm(&inkwell::targets::InitializationConfig::default());
}

///
/// Implemented by items which are translated into LLVM IR.
///
pub trait WriteLLVM<D>
where
    D: Dependency,
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
    D: Dependency,
{
    fn into_llvm(self, _context: &mut Context<D>) -> anyhow::Result<()> {
        Ok(())
    }
}
