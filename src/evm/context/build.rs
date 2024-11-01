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
    /// The project metadata hash.
    pub metadata_hash: Option<Vec<u8>>,
}

impl Build {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(bytecode: Vec<u8>, metadata_hash: Option<Vec<u8>>) -> Self {
        Self {
            bytecode,
            metadata_hash,
        }
    }
}
