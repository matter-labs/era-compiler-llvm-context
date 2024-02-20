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
    /// Whether the contract minimal proxy has been used.
    is_minimal_proxy_used: bool,
}

impl VyperData {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(immutables_size: usize, is_minimal_proxy_used: bool) -> Self {
        Self {
            immutables_size,
            is_minimal_proxy_used,
        }
    }

    ///
    /// Returns the size of the immutables data of the contract.
    ///
    pub fn immutables_size(&self) -> usize {
        self.immutables_size
    }

    ///
    /// Sets the minimal proxy usage flag.
    ///
    pub fn set_is_minimal_proxy_used(&mut self) {
        self.is_minimal_proxy_used = true;
    }

    ///
    /// Returns the minimal proxy usage flag.
    ///
    pub fn is_minimal_proxy_used(&self) -> bool {
        self.is_minimal_proxy_used
    }
}
