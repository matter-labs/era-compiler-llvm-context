//!
//! The LLVM module build.
//!

use std::collections::BTreeMap;

///
/// The LLVM module build.
///
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Build {
    /// Bytecode.
    pub bytecode: Vec<u8>,
    /// Bytecode hash. Only available after linking.
    pub bytecode_hash: Option<[u8; era_compiler_common::BYTE_LENGTH_FIELD]>,
    /// Project metadata.
    pub metadata: Vec<u8>,
    /// Hash-to-full-path mapping of the contract factory dependencies.
    pub factory_dependencies: BTreeMap<String, String>,
    /// Text assembly.
    pub assembly: Option<String>,
}

impl Build {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(bytecode: Vec<u8>, metadata: Vec<u8>, assembly: Option<String>) -> Self {
        Self {
            bytecode,
            bytecode_hash: None,
            metadata,
            factory_dependencies: BTreeMap::new(),
            assembly,
        }
    }
}
