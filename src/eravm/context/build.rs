//!
//! The LLVM module build.
//!

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

///
/// The LLVM module build.
///
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Build {
    /// The bytecode.
    pub bytecode: Vec<u8>,
    /// The bytecode hash.
    pub bytecode_hash: [u8; era_compiler_common::BYTE_LENGTH_FIELD],
    /// The project metadata hash.
    pub metadata_hash: Option<[u8; era_compiler_common::BYTE_LENGTH_FIELD]>,
    /// The hash-to-full-path mapping of the contract factory dependencies.
    pub factory_dependencies: BTreeMap<String, String>,
    /// The text assembly.
    pub assembly: Option<String>,
}

impl Build {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        bytecode: Vec<u8>,
        bytecode_hash: [u8; era_compiler_common::BYTE_LENGTH_FIELD],
        metadata_hash: Option<[u8; era_compiler_common::BYTE_LENGTH_FIELD]>,
        assembly: Option<String>,
    ) -> Self {
        Self {
            bytecode,
            bytecode_hash,
            metadata_hash,
            factory_dependencies: BTreeMap::new(),
            assembly,
        }
    }
}
