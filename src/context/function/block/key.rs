//!
//! The LLVM IR generator function block key.
//!

///
/// The LLVM IR generator function block key.
///
/// Is only relevant to the EVM legacy assembly.
///
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key {
    /// The block code type.
    pub code_segment: era_compiler_common::CodeSegment,
    /// The block tag.
    pub tag: num::BigUint,
}

impl Key {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(code_segment: era_compiler_common::CodeSegment, tag: num::BigUint) -> Self {
        Self { code_segment, tag }
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}_{}",
            match self.code_segment {
                era_compiler_common::CodeSegment::Deploy => "dt",
                era_compiler_common::CodeSegment::Runtime => "rt",
            },
            self.tag
        )
    }
}
