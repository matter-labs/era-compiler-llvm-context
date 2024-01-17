//!
//! The LLVM module build.
//!

use std::collections::BTreeMap;
use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;

///
/// The LLVM module build.
///
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Build {
    /// The text assembly.
    pub assembly_text: String,
    /// The metadata hash.
    pub metadata_hash: Option<[u8; compiler_common::BYTE_LENGTH_FIELD]>,
    /// The binary bytecode.
    pub bytecode: Vec<u8>,
    /// The bytecode hash.
    pub bytecode_hash: String,
    /// The hash-to-full-path mapping of the contract factory dependencies.
    pub factory_dependencies: BTreeMap<String, String>,
    /// The missing deployable libraries.
    pub missing_libraries: HashSet<String>,
}

impl Build {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        assembly_text: String,
        metadata_hash: Option<[u8; compiler_common::BYTE_LENGTH_FIELD]>,
        bytecode: Vec<u8>,
        bytecode_hash: String,
        missing_libraries: HashSet<String>,
    ) -> Self {
        Self {
            assembly_text,
            metadata_hash,
            bytecode,
            bytecode_hash,
            factory_dependencies: BTreeMap::new(),
            missing_libraries,
        }
    }
}
