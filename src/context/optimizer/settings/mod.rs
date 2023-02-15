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
    /// Whether to run the inliner.
    pub is_inliner_enabled: bool,
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
        is_inliner_enabled: bool,
        level_back_end: inkwell::OptimizationLevel,
    ) -> Self {
        Self {
            level_middle_end,
            level_middle_end_size,
            is_inliner_enabled,
            level_back_end,
        }
    }

    ///
    /// Returns the settings without optimizations.
    ///
    pub fn none() -> Self {
        Self::new(
            inkwell::OptimizationLevel::None,
            SizeLevel::Zero,
            false,
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
            true,
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
            true,
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
        .cartesian_product(vec![/*false, */ true])
        .cartesian_product(vec![
            inkwell::OptimizationLevel::None,
            inkwell::OptimizationLevel::Less,
            inkwell::OptimizationLevel::Default,
            inkwell::OptimizationLevel::Aggressive,
        ])
        .map(
            |((optimization_level_middle, is_inliner_enabled), optimization_level_back)| {
                Self::new(
                    optimization_level_middle,
                    SizeLevel::Zero,
                    is_inliner_enabled,
                    optimization_level_back,
                )
            },
        )
        .collect();
        combinations.push(Self::new(
            inkwell::OptimizationLevel::Default,
            SizeLevel::S,
            true,
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
            "M{}I{}B{}",
            self.middle_end_as_string(),
            if self.is_inliner_enabled { '+' } else { '-' },
            self.level_back_end as u8,
        )
    }
}
