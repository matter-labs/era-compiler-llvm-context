//!
//! The LLVM IR Solidity data trait.
//!

use std::collections::BTreeSet;

///
/// The LLVM IR Solidity data trait.
///
pub trait ISolidityData {
    ///
    /// Returns all runtime code offsets for the specified `id`.
    ///
    fn offsets(&mut self, id: &str) -> Option<BTreeSet<u64>>;

    ///
    /// Sets the spill area size.
    ///
    /// Only used in EVM to avoid stack-too-deep errors.
    ///
    fn set_spill_area_size(&mut self, _size: u64) {}

    ///
    /// Gets the spill area size.
    ///
    /// Only used in EVM to avoid stack-too-deep errors.
    ///
    fn spill_area_size(&self) -> Option<u64> {
        None
    }
}
