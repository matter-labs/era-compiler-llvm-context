//!
//! The LLVM module build.
//!

///
/// The LLVM module build.
///
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Build {
    /// The bytecode.
    pub bytecode: Vec<u8>,
    /// The bytecode hash. Only available after linking.
    pub bytecode_hash: Option<[u8; era_compiler_common::BYTE_LENGTH_FIELD]>,
    /// The project metadata hash.
    pub metadata_hash: Option<Vec<u8>>,
    /// The text assembly.
    pub assembly: Option<String>,
}

impl Build {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        bytecode: Vec<u8>,
        bytecode_hash: Option<[u8; era_compiler_common::BYTE_LENGTH_FIELD]>,
        metadata_hash: Option<Vec<u8>>,
        assembly: Option<String>,
    ) -> Self {
        Self {
            bytecode,
            bytecode_hash,
            metadata_hash,
            assembly,
        }
    }
}
