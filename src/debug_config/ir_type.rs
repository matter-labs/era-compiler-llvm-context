//!
//! The debug IR type.
//!

///
/// The debug IR type.
///
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IRType {
    /// Whether to dump the Yul code.
    Yul,
    /// Whether to dump the EVM legacy assembly code.
    EVMLA,
    /// Whether to dump the Ethereal IR code.
    EthIR,
    /// Whether to dump the Vyper LLL IR code.
    LLL,
    /// Whether to dump the LLVM IR code.
    LLVM,
    /// Whether to dump the assembly code.
    Assembly,
}

impl IRType {
    ///
    /// Returns the file extension for the specified IR.
    ///
    pub fn file_extension(&self) -> &'static str {
        match self {
            Self::Yul => compiler_common::EXTENSION_YUL,
            Self::EthIR => compiler_common::EXTENSION_ETHIR,
            Self::EVMLA => compiler_common::EXTENSION_EVMLA,
            Self::LLL => compiler_common::EXTENSION_LLL,
            Self::LLVM => compiler_common::EXTENSION_LLVM_SOURCE,
            Self::Assembly => compiler_common::EXTENSION_ERAVM_ASSEMBLY,
        }
    }
}
