//!
//! The LLVM optimizer settings size level.
//!

use serde::Deserialize;
use serde::Serialize;

///
/// The LLVM optimizer settings size level.
///
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum SizeLevel {
    /// No size optimizations.
    Zero,
    /// The default size optimizations.
    S,
    /// The aggresize size optimizations.
    Z,
}

impl From<SizeLevel> for u32 {
    fn from(level: SizeLevel) -> Self {
        match level {
            SizeLevel::Zero => 0,
            SizeLevel::S => 1,
            SizeLevel::Z => 2,
        }
    }
}
