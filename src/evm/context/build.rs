//!
//! The LLVM module build.
//!

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
}

impl Build {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        assembly_text: String,
        metadata_hash: Option<[u8; compiler_common::BYTE_LENGTH_FIELD]>,
        bytecode: Vec<u8>,
    ) -> Self {
        Self {
            assembly_text,
            metadata_hash,
            bytecode,
        }
    }
}
