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
    /// Unlinked symbols.
    pub unlinked_symbols: Option<BTreeMap<String, Vec<u64>>>,
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
        unlinked_symbols: Option<BTreeMap<String, Vec<u64>>>,
        warnings: Vec<Warning>,
    ) -> Self {
        Self {
            bytecode,
            assembly,
            immutables,
            unlinked_symbols,
            warnings,
        }
    }
}
