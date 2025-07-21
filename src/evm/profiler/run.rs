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
    /// Start time.
    pub start_time: Instant,
    /// Recorded duration.
    pub duration: Option<Duration>,
}

impl Default for Run {
    fn default() -> Self {
        Run {
            start_time: Instant::now(),
            duration: None,
        }
    }
}

impl Run {
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
