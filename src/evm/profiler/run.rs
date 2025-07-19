//!
//! Compiler pipeline profiler run.
//!

use std::time::Duration;
use std::time::Instant;

///
/// Compiler pipeline profiler run.
///
#[derive(Debug)]
pub struct Run {
    /// The run level.
    /// Used for granularity in profiling, where a higher level indicates a broader scope.
    pub level: usize,
    /// Start time.
    pub start_time: Instant,
    /// Recorded duration.
    pub duration: Option<Duration>,
}

impl Run {
    ///
    /// Creates a new run with the given name and level.
    ///
    pub fn new(level: usize) -> Self {
        Run {
            level,
            start_time: Instant::now(),
            duration: None,
        }
    }

    ///
    /// Records the duration of the run.
    ///
    pub fn finish(&mut self) {
        assert!(
            self.duration.is_none(),
            "Duration has already been recorded"
        );

        self.duration = Some(self.start_time.elapsed());
    }
}

impl std::fmt::Display for Run {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let duration = self
            .duration
            .as_ref()
            .expect("Duration has not been recorded yet");

        write!(f, "{duration:?}")?;
        Ok(())
    }
}
