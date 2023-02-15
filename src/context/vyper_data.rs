//!
//! The LLVM IR generator Vyper data.
//!

///
/// The LLVM IR generator Vyper data.
///
/// Describes some data that is only relevant to Vyper.
///
#[derive(Debug)]
pub struct VyperData {
    /// The immutables size tracker. Stores the size in bytes.
    /// Does not take into account the size of the indexes.
    immutables_size: usize,
}

impl VyperData {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(immutables_size: usize) -> Self {
        Self { immutables_size }
    }

    ///
    /// Returns the size of the immutables data of the contract.
    ///
    pub fn immutables_size(&self) -> usize {
        self.immutables_size
    }
}
