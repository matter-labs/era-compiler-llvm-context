//!
//! The LLVM function block EVM legacy assembly data.
//!

///
/// The LLVM function block EVM legacy assembly data.
///
/// Describes some data that is only relevant to the EVM legacy assembly.
///
#[derive(Debug, Clone)]
pub struct EVMLAData {
    /// The initial hashes of the allowed stack states.
    pub stack_hashes: Vec<[u8; era_compiler_common::BYTE_LENGTH_FIELD]>,
}

impl EVMLAData {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(stack_hashes: Vec<[u8; era_compiler_common::BYTE_LENGTH_FIELD]>) -> Self {
        Self { stack_hashes }
    }
}
