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
