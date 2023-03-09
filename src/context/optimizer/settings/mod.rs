//!
//! The LLVM optimizer settings.
//!

pub mod size_level;

use itertools::Itertools;

use self::size_level::SizeLevel;

///
/// The LLVM optimizer settings.
///
#[derive(Debug, Clone)]
pub struct Settings {
    /// The middle-end optimization level.
    pub level_middle_end: inkwell::OptimizationLevel,
    /// The middle-end size optimization level.
    pub level_middle_end_size: SizeLevel,
    /// The back-end optimization level.
    pub level_back_end: inkwell::OptimizationLevel,
}

impl Settings {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        level_middle_end: inkwell::OptimizationLevel,
        level_middle_end_size: SizeLevel,
        level_back_end: inkwell::OptimizationLevel,
    ) -> Self {
        Self {
            level_middle_end,
            level_middle_end_size,
            level_back_end,
        }
    }

    ///
    /// Creates settings from a CLI optimization parameter.
    ///
    pub fn try_from_cli(value: char) -> anyhow::Result<Self> {
        Ok(match value {
            '0' => Self {
                level_middle_end: inkwell::OptimizationLevel::None,
                level_middle_end_size: SizeLevel::Zero,
                level_back_end: inkwell::OptimizationLevel::None,
            },
            '1' => Self {
                level_middle_end: inkwell::OptimizationLevel::Less,
                level_middle_end_size: SizeLevel::Zero,
                level_back_end: inkwell::OptimizationLevel::Less,
            },
            '2' => Self {
                level_middle_end: inkwell::OptimizationLevel::Default,
                level_middle_end_size: SizeLevel::Zero,
                level_back_end: inkwell::OptimizationLevel::Default,
            },
            '3' => Self {
                level_middle_end: inkwell::OptimizationLevel::Aggressive,
                level_middle_end_size: SizeLevel::Zero,
                level_back_end: inkwell::OptimizationLevel::Aggressive,
            },
            's' => Self {
                level_middle_end: inkwell::OptimizationLevel::Default,
                level_middle_end_size: SizeLevel::S,
                level_back_end: inkwell::OptimizationLevel::Aggressive,
            },
            'z' => Self {
                level_middle_end: inkwell::OptimizationLevel::Default,
                level_middle_end_size: SizeLevel::Z,
                level_back_end: inkwell::OptimizationLevel::Aggressive,
            },
            char => anyhow::bail!("Unexpected optimization option '{}'", char),
        })
    }

    ///
    /// Returns the settings without optimizations.
    ///
    pub fn none() -> Self {
        Self::new(
            inkwell::OptimizationLevel::None,
            SizeLevel::Zero,
            inkwell::OptimizationLevel::None,
        )
    }

    ///
    /// Returns the settings for the optimal number of VM execution cycles.
    ///
    pub fn cycles() -> Self {
        Self::new(
            inkwell::OptimizationLevel::Aggressive,
            SizeLevel::Zero,
            inkwell::OptimizationLevel::Aggressive,
        )
    }

    ///
    /// Returns the settings for the optimal size.
    ///
    pub fn size() -> Self {
        Self::new(
            inkwell::OptimizationLevel::Default,
            SizeLevel::Z,
            inkwell::OptimizationLevel::Aggressive,
        )
    }

    ///
    /// Returns the middle-end optimization parameter as string.
    ///
    pub fn middle_end_as_string(&self) -> String {
        match self.level_middle_end_size {
            SizeLevel::Zero => (self.level_middle_end as u8).to_string(),
            SizeLevel::S => 's'.to_string(),
            SizeLevel::Z => 'z'.to_string(),
        }
    }

    ///
    /// Checks whether there are middle-end optimizations enabled.
    ///
    pub fn is_middle_end_enabled(&self) -> bool {
        self.level_middle_end != inkwell::OptimizationLevel::None
            || self.level_middle_end_size != SizeLevel::Zero
    }

    ///
    /// Returns all possible combinations of the optimizer settings.
    ///
    pub fn combinations() -> Vec<Self> {
        let mut combinations: Vec<Self> = vec![
            inkwell::OptimizationLevel::None,
            inkwell::OptimizationLevel::Less,
            inkwell::OptimizationLevel::Default,
            inkwell::OptimizationLevel::Aggressive,
        ]
        .into_iter()
        .cartesian_product(vec![
            inkwell::OptimizationLevel::None,
            inkwell::OptimizationLevel::Less,
            inkwell::OptimizationLevel::Default,
            inkwell::OptimizationLevel::Aggressive,
        ])
        .map(|(optimization_level_middle, optimization_level_back)| {
            Self::new(
                optimization_level_middle,
                SizeLevel::Zero,
                optimization_level_back,
            )
        })
        .collect();
        combinations.push(Self::new(
            inkwell::OptimizationLevel::Default,
            SizeLevel::S,
            inkwell::OptimizationLevel::Aggressive,
        ));
        combinations.push(Self::size());
        combinations
    }
}

impl std::fmt::Display for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "M{}B{}",
            self.middle_end_as_string(),
            self.level_back_end as u8,
        )
    }
}
