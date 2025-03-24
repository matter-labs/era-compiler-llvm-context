//!
//! The LLVM context constants.
//!

/// The LLVM framework version.
pub const LLVM_VERSION: semver::Version = semver::Version::new(17, 0, 4);

/// The entry function name.
pub const ENTRY_FUNCTION_NAME: &str = "__entry";

/// The deployed Yul object identifier suffix.
pub static YUL_OBJECT_DEPLOYED_SUFFIX: &str = "_deployed";

/// Library deploy address Yul identifier.
pub static LIBRARY_DEPLOY_ADDRESS_TAG: &str = "library_deploy_address";
