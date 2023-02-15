//!
//! The LLVM module build.
//!

use std::collections::BTreeMap;

///
/// The LLVM module build.
///
#[derive(Debug)]
pub struct Build {
    /// The zkEVM text assembly.
    pub assembly_text: String,
    /// The zkEVM binary assembly.
    pub assembly: zkevm_assembly::Assembly,
    /// The zkEVM binary bytecode.
    pub bytecode: Vec<u8>,
    /// The zkEVM bytecode hash.
    pub hash: String,
    /// The hash-to-full-path mapping of the contract factory dependencies.
    pub factory_dependencies: BTreeMap<String, String>,
}

impl Build {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        assembly_text: String,
        assembly: zkevm_assembly::Assembly,
        bytecode: Vec<u8>,
        hash: String,
    ) -> Self {
        Self {
            assembly_text,
            assembly,
            bytecode,
            hash,
            factory_dependencies: BTreeMap::new(),
        }
    }
}
