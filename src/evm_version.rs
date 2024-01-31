//!
//! The EVM version.
//!

use serde::Deserialize;
use serde::Serialize;

///
/// The EVM version.
///
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "camelCase")]
pub enum EVMVersion {
    /// The corresponding EVM version.
    #[serde(rename = "homestead")]
    Homestead,
    /// The corresponding EVM version.
    #[serde(rename = "tangerineWhistle")]
    TangerineWhistle,
    /// The corresponding EVM version.
    #[serde(rename = "spuriousDragon")]
    SpuriousDragon,
    /// The corresponding EVM version.
    #[serde(rename = "byzantium")]
    Byzantium,
    /// The corresponding EVM version.
    #[serde(rename = "constantinople")]
    Constantinople,
    /// The corresponding EVM version.
    #[serde(rename = "petersburg")]
    Petersburg,
    /// The corresponding EVM version.
    #[serde(rename = "istanbul")]
    Istanbul,
    /// The corresponding EVM version.
    #[serde(rename = "berlin")]
    Berlin,
    /// The corresponding EVM version.
    #[serde(rename = "london")]
    London,
    /// The corresponding EVM version.
    #[serde(rename = "paris")]
    Paris,
    /// The corresponding EVM version.
    #[serde(rename = "shanghai")]
    Shanghai,
    /// The corresponding EVM version.
    #[serde(rename = "cancun")]
    Cancun,
    /// The corresponding EVM version.
    #[serde(rename = "atlantis")]
    Atlantis,
    /// The corresponding EVM version.
    #[serde(rename = "agharta")]
    Agharta,
}

impl Default for EVMVersion {
    fn default() -> Self {
        Self::Shanghai
    }
}

impl TryFrom<&str> for EVMVersion {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "homestead" => Self::Homestead,
            "tangerineWhistle" => Self::TangerineWhistle,
            "spuriousDragon" => Self::SpuriousDragon,
            "byzantium" => Self::Byzantium,
            "constantinople" => Self::Constantinople,
            "petersburg" => Self::Petersburg,
            "istanbul" => Self::Istanbul,
            "berlin" => Self::Berlin,
            "london" => Self::London,
            "paris" => Self::Paris,
            "shanghai" => Self::Shanghai,
            "cancun" => Self::Cancun,
            "atlantis" => Self::Atlantis,
            "agharta" => Self::Agharta,
            _ => anyhow::bail!("Invalid EVM version: {}", value),
        })
    }
}

impl std::fmt::Display for EVMVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Homestead => write!(f, "homestead"),
            Self::TangerineWhistle => write!(f, "tangerineWhistle"),
            Self::SpuriousDragon => write!(f, "spuriousDragon"),
            Self::Byzantium => write!(f, "byzantium"),
            Self::Constantinople => write!(f, "constantinople"),
            Self::Petersburg => write!(f, "petersburg"),
            Self::Istanbul => write!(f, "istanbul"),
            Self::Berlin => write!(f, "berlin"),
            Self::London => write!(f, "london"),
            Self::Paris => write!(f, "paris"),
            Self::Shanghai => write!(f, "shanghai"),
            Self::Cancun => write!(f, "cancun"),
            Self::Atlantis => write!(f, "atlantis"),
            Self::Agharta => write!(f, "agharta"),
        }
    }
}
