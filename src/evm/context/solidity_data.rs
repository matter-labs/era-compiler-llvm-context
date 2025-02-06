//!
//! The LLVM IR generator Solidity data.
//!

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::context::traits::solidity_data::ISolidityData;

///
/// The LLVM IR generator Solidity data.
///
/// Describes some data that is only relevant to Solidity.
///
#[derive(Debug, Default)]
pub struct SolidityData {
    /// The immutables identifier-to-offset mapping.
    immutables: BTreeMap<String, BTreeSet<u64>>,
}

impl ISolidityData for SolidityData {
    fn offsets(&self, id: &str) -> Option<&BTreeSet<u64>> {
        self.immutables.get(id)
    }
}

impl SolidityData {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(immutables: BTreeMap<String, BTreeSet<u64>>) -> Self {
        Self { immutables }
    }
}
