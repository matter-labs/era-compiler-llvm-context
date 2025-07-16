//!
//! The LLVM module build.
//!

use std::collections::BTreeMap;
use std::collections::BTreeSet;

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
    /// Mapping with immutables.
    pub immutables: Option<BTreeMap<String, BTreeSet<u64>>>,
    /// Whether the size fallback has been activated.
    pub is_size_fallback: bool,
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
        immutables: Option<BTreeMap<String, BTreeSet<u64>>>,
        is_size_fallback: bool,
        warnings: Vec<Warning>,
    ) -> Self {
        Self {
            bytecode,
            assembly,
            immutables,
            is_size_fallback,
            warnings,
        }
    }
}
