//!
//! Compiler pipeline profiler.
//!

pub mod run;

use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::optimizer::settings::Settings as OptimizerSettings;

use self::run::Run;

///
/// Compiler pipeline profiler.
///
#[derive(Debug, Default)]
pub struct Profiler {
    /// Indexed map of timing entries.
    pub timings: IndexMap<String, Rc<RefCell<Run>>>,
}

impl Profiler {
    ///
    /// Starts a new run for a generic part of the pipeline.
    ///
    pub fn start(&mut self, description: &str) -> Rc<RefCell<Run>> {
        let run_name = description.to_owned();
        assert!(
            !self.timings.contains_key(run_name.as_str()),
            "Translation unit run `{run_name}` already exists"
        );

        self.start_run(run_name, 0)
    }

    ///
    /// Starts a new run for an EVM translation unit.
    ///
    pub fn start_evm_contract(&mut self, full_path: &str) -> Rc<RefCell<Run>> {
        let run_name = full_path.to_owned();
        assert!(
            !self.timings.contains_key(run_name.as_str()),
            "Translation unit run `{run_name}` already exists"
        );

        self.start_run(run_name, 1)
    }

    ///
    /// Starts a new run for an EVM translation unit.
    ///
    pub fn start_evm_translation_unit(
        &mut self,
        full_path: &str,
        code_segment: era_compiler_common::CodeSegment,
        description: &str,
        optimizer_settings: &OptimizerSettings,
    ) -> Rc<RefCell<Run>> {
        let spill_area_description = format!(
            "SpillArea({})",
            optimizer_settings.spill_area_size().unwrap_or_default()
        );
        let run_name = format!(
            "{full_path}:{code_segment}/{description}/{optimizer_settings}/{spill_area_description}",
        );
        assert!(
            !self.timings.contains_key(run_name.as_str()),
            "Translation unit run `{run_name}` already exists"
        );

        self.start_run(run_name, 2)
    }

    ///
    /// Starts a new run with the given name.
    ///
    fn start_run(&mut self, name: String, level: usize) -> Rc<RefCell<Run>> {
        assert!(
            !self.timings.contains_key(name.as_str()),
            "Run `{name}` already exists"
        );

        let run = Rc::new(RefCell::new(Run::new(level)));
        self.timings.insert(name, run.clone());
        run
    }
}

impl std::fmt::Display for Profiler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (name, run) in self.timings.iter() {
            let run = run.borrow();
            writeln!(f, "{}{name}: {run}", "    ".repeat(run.level))?;
        }
        Ok(())
    }
}
