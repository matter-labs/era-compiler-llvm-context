//!
//! The LLVM context constants.
//!

/// The EraVM version.
pub const ZKEVM_VERSION: semver::Version = semver::Version::new(1, 3, 2);

/// The deployed Yul object identifier suffix.
pub static YUL_OBJECT_DEPLOYED_SUFFIX: &str = "_deployed";

/// The heap memory pointer pointer global variable name.
pub static GLOBAL_HEAP_MEMORY_POINTER: &str = "memory_pointer";

/// The calldata pointer global variable name.
pub static GLOBAL_CALLDATA_POINTER: &str = "ptr_calldata";

/// The calldata size pointer global variable name.
pub static GLOBAL_CALLDATA_SIZE: &str = "calldatasize";

/// The return data pointer global variable name.
pub static GLOBAL_RETURN_DATA_POINTER: &str = "ptr_return_data";

/// The return data size pointer global variable name.
pub static GLOBAL_RETURN_DATA_SIZE: &str = "returndatasize";

/// The decommit pointer global variable name.
pub static GLOBAL_DECOMMIT_POINTER: &str = "ptr_decommit";

/// The call flags global variable name.
pub static GLOBAL_CALL_FLAGS: &str = "call_flags";

/// The extra ABI data global variable name.
pub static GLOBAL_EXTRA_ABI_DATA: &str = "extra_abi_data";

/// The active pointer array global variable name.
pub static GLOBAL_ACTIVE_POINTER_ARRAY: &str = "ptr_active";

/// The constant array global variable name prefix.
pub static GLOBAL_CONST_ARRAY_PREFIX: &str = "const_array_";

/// The global verbatim getter identifier prefix.
pub static GLOBAL_VERBATIM_GETTER_PREFIX: &str = "get_global::";

/// The number of available active pointers.
pub const AVAILABLE_ACTIVE_POINTERS_NUMBER: usize = 16;

/// The external call data offset in the auxiliary heap.
pub const HEAP_AUX_OFFSET_EXTERNAL_CALL: u64 = 0;

/// The constructor return data offset in the auxiliary heap.
pub const HEAP_AUX_OFFSET_CONSTRUCTOR_RETURN_DATA: u64 =
    8 * (era_compiler_common::BYTE_LENGTH_FIELD as u64);

/// The number of the extra ABI data arguments.
pub const EXTRA_ABI_DATA_SIZE: usize = 10;

/// The `create` method deployer signature.
pub static DEPLOYER_SIGNATURE_CREATE: &str = "create(bytes32,bytes32,bytes)";

/// The `create2` method deployer signature.
pub static DEPLOYER_SIGNATURE_CREATE2: &str = "create2(bytes32,bytes32,bytes)";

/// The absence of system call bit.
pub const NO_SYSTEM_CALL_BIT: bool = false;

/// The system call bit.
pub const SYSTEM_CALL_BIT: bool = true;

/// The default disassembler mode.
pub const DISASSEMBLER_DEFAULT_MODE: u64 = 1;

///
/// The deployer call header size that consists of:
/// - selector (4 bytes)
/// - salt (32 bytes)
/// - bytecode hash (32 bytes)
/// - constructor arguments offset (32 bytes)
/// - constructor arguments length (32 bytes)
///
pub const DEPLOYER_CALL_HEADER_SIZE: usize =
    era_compiler_common::BYTE_LENGTH_X32 + (era_compiler_common::BYTE_LENGTH_FIELD * 4);
