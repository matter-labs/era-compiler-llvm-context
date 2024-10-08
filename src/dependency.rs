//!
//! The dependency trait.
//!

///
/// Implemented by entities managing project dependencies.
///
pub trait Dependency {
    ///
    /// Gets the data of the specified dependency.
    ///
    fn get(&self, path: &str) -> anyhow::Result<String>;

    ///
    /// Resolves a full contract path.
    ///
    fn resolve_path(&self, identifier: &str) -> anyhow::Result<String>;

    ///
    /// Resolves a library address.
    ///
    fn resolve_library(&self, path: &str) -> Option<String>;
}

///
/// The dummy dependency entity.
///
#[derive(Debug, Default, Clone)]
pub struct DummyDependency {}

impl Dependency for DummyDependency {
    fn get(&self, _path: &str) -> anyhow::Result<String> {
        Ok(String::new())
    }

    ///
    /// Resolves a full contract path.
    ///
    fn resolve_path(&self, _identifier: &str) -> anyhow::Result<String> {
        Ok(String::new())
    }

    ///
    /// Resolves a library address.
    ///
    fn resolve_library(&self, _path: &str) -> Option<String> {
        None
    }
}
