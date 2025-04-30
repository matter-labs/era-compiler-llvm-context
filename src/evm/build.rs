//!
//! The LLVM module build.
//!

use crate::evm::warning::Warning;

///
/// The LLVM module build.
///
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Build {
    /// Bytecode.
    pub bytecode: Option<Vec<u8>>,
    /// Text assembly.
    pub assembly: Option<String>,
    /// Warnings produced during compilation.
    pub warnings: Vec<Warning>,
}

impl Build {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        bytecode: Option<Vec<u8>>,
        assembly: Option<String>,
        warnings: Vec<Warning>,
    ) -> Self {
        Self {
            bytecode,
            assembly,
            warnings,
        }
    }
}
