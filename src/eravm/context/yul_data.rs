//!
//! The LLVM IR generator Yul data.
//!

///
/// The LLVM IR generator Yul data.
///
/// Describes some data that is only relevant to Yul.
///
#[derive(Debug, Default)]
pub struct YulData {
    /// The system mode flag.
    /// The call simulations only work if this mode is enabled.
    is_system_mode: bool,
}

impl YulData {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(is_system_mode: bool) -> Self {
        Self { is_system_mode }
    }

    ///
    /// Whether the system mode is enabled.
    ///
    pub fn is_system_mode(&self) -> bool {
        self.is_system_mode
    }
}
