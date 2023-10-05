//!
//! The LLVM target.
//!

use std::str::FromStr;

///
/// The LLVM target.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// The EraVM target.
    EraVM,
    /// The EVM target.
    EVM,
}

impl Target {
    ///
    /// Returns the target name.
    ///
    pub fn name(&self) -> &str {
        match self {
            Self::EraVM => "eravm",
            Self::EVM => "evm",
        }
    }

    ///
    /// Returns the target triple.
    ///
    pub fn triple(&self) -> &str {
        match self {
            Self::EraVM => "eravm-unknown-unknown",
            Self::EVM => "evm-unknown-unknown",
        }
    }

    ///
    /// Returns the target production name.
    ///
    pub fn production_name(&self) -> &str {
        match self {
            Self::EraVM => "EraVM",
            Self::EVM => "EVM",
        }
    }
}

impl FromStr for Target {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "eravm" => Ok(Self::EraVM),
            "evm" => Ok(Self::EVM),
            _ => Err(anyhow::anyhow!(
                "Unknown target `{}`. Supported targets: {:?}",
                string,
                vec![Self::EraVM, Self::EVM]
            )),
        }
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Target::EraVM => write!(f, "eravm"),
            Target::EVM => write!(f, "evm"),
        }
    }
}
