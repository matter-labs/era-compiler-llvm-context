//!
//! The LLVM attribute.
//!

pub mod memory;

///
/// The LLVM attribute.
///
/// In order to check the real order in a new major version of LLVM, find the `Attributes.inc` file
/// inside of the LLVM build directory. This order is actually generated during the building.
///
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum Attribute {
    /// Unused.
    Unused = 0,
    /// The eponymous LLVM attribute.
    AllocAlign = 1,
    /// The eponymous LLVM attribute.
    AllocatedPointer = 2,
    /// The eponymous LLVM attribute.
    AlwaysInline = 3,
    /// The eponymous LLVM attribute.
    Builtin = 4,
    /// The eponymous LLVM attribute.
    Cold = 5,
    /// The eponymous LLVM attribute.
    Convergent = 6,
    /// The eponymous LLVM attribute.
    DisableSanitizerInstrumentation = 7,
    /// The eponymous LLVM attribute.
    FnRetThunkExtern = 8,
    /// The eponymous LLVM attribute.
    Hot = 9,
    /// The eponymous LLVM attribute.
    ImmArg = 10,
    /// The eponymous LLVM attribute.
    InReg = 11,
    /// The eponymous LLVM attribute.
    InlineHint = 12,
    /// The eponymous LLVM attribute.
    JumpTable = 13,
    /// The eponymous LLVM attribute.
    MinSize = 14,
    /// The eponymous LLVM attribute.
    MustProgress = 15,
    /// The eponymous LLVM attribute.
    Naked = 16,
    /// The eponymous LLVM attribute.
    Nest = 17,
    /// The eponymous LLVM attribute.
    NoAlias = 18,
    /// The eponymous LLVM attribute.
    NoBuiltin = 19,
    /// The eponymous LLVM attribute.
    NoCallback = 20,
    /// The eponymous LLVM attribute.
    NoCapture = 21,
    /// The eponymous LLVM attribute.
    NoCfCheck = 22,
    /// The eponymous LLVM attribute.
    NoDuplicate = 23,
    /// The eponymous LLVM attribute.
    NoFree = 24,
    /// The eponymous LLVM attribute.
    NoImplicitFloat = 25,
    /// The eponymous LLVM attribute.
    NoInline = 26,
    /// The eponymous LLVM attribute.
    NoMerge = 27,
    /// The eponymous LLVM attribute.
    NoProfile = 28,
    /// The eponymous LLVM attribute.
    NoRecurse = 29,
    /// The eponymous LLVM attribute.
    NoRedZone = 30,
    /// The eponymous LLVM attribute.
    NoReturn = 31,
    /// The eponymous LLVM attribute.
    NoSanitizeBounds = 32,
    /// The eponymous LLVM attribute.
    NoSanitizeCoverage = 33,
    /// The eponymous LLVM attribute.
    NoSync = 34,
    /// The eponymous LLVM attribute.
    NoUndef = 35,
    /// The eponymous LLVM attribute.
    NoUnwind = 36,
    /// The eponymous LLVM attribute.
    NonLazyBind = 37,
    /// The eponymous LLVM attribute.
    NonNull = 38,
    /// The eponymous LLVM attribute.
    NullPointerIsValid = 39,
    /// The eponymous LLVM attribute.
    OptForFuzzing = 40,
    /// The eponymous LLVM attribute.
    OptimizeForSize = 41,
    /// The eponymous LLVM attribute.
    OptimizeNone = 42,
    /// The eponymous LLVM attribute.
    PresplitCoroutine = 43,
    /// The eponymous LLVM attribute.
    ReadNone = 44,
    /// The eponymous LLVM attribute.
    ReadOnly = 45,
    /// The eponymous LLVM attribute.
    Returned = 46,
    /// The eponymous LLVM attribute.
    ReturnsTwice = 47,
    /// The eponymous LLVM attribute.
    SExt = 48,
    /// The eponymous LLVM attribute.
    SafeStack = 49,
    /// The eponymous LLVM attribute.
    SanitizeAddress = 50,
    /// The eponymous LLVM attribute.
    SanitizeHWAddress = 51,
    /// The eponymous LLVM attribute.
    SanitizeMemTag = 52,
    /// The eponymous LLVM attribute.
    SanitizeMemory = 53,
    /// The eponymous LLVM attribute.
    SanitizeThread = 54,
    /// The eponymous LLVM attribute.
    ShadowCallStack = 55,
    /// The eponymous LLVM attribute.
    SkipProfile = 56,
    /// The eponymous LLVM attribute.
    Speculatable = 57,
    /// The eponymous LLVM attribute.
    SpeculativeLoadHardening = 58,
    /// The eponymous LLVM attribute.
    StackProtect = 59,
    /// The eponymous LLVM attribute.
    StackProtectReq = 60,
    /// The eponymous LLVM attribute.
    StackProtectStrong = 61,
    /// The eponymous LLVM attribute.
    StrictFP = 62,
    /// The eponymous LLVM attribute.
    SwiftAsync = 63,
    /// The eponymous LLVM attribute.
    SwiftError = 64,
    /// The eponymous LLVM attribute.
    SwiftSelf = 65,
    /// The eponymous LLVM attribute.
    WillReturn = 66,
    /// The eponymous LLVM attribute.
    WriteOnly = 67,
    /// The eponymous LLVM attribute.
    ZExt = 68,
    /// The eponymous LLVM attribute.
    ByRef = 69,
    /// The eponymous LLVM attribute.
    ByVal = 70,
    /// The eponymous LLVM attribute.
    ElementType = 71,
    /// The eponymous LLVM attribute.
    InAlloca = 72,
    /// The eponymous LLVM attribute.
    Preallocated = 73,
    /// The eponymous LLVM attribute.
    StructRet = 74,
    /// The eponymous LLVM attribute.
    Alignment = 75,
    /// The eponymous LLVM attribute.
    AllocKind = 76,
    /// The eponymous LLVM attribute.
    AllocSize = 77,
    /// The eponymous LLVM attribute.
    Dereferenceable = 78,
    /// The eponymous LLVM attribute.
    DereferenceableOrNull = 79,
    /// The eponymous LLVM attribute.
    Memory = 80,
    /// The eponymous LLVM attribute.
    NoFPClass = 81,
    /// The eponymous LLVM attribute.
    StackAlignment = 82,
    /// The eponymous LLVM attribute.
    UWTable = 83,
    /// The eponymous LLVM attribute.
    VScaleRange = 84,
}

impl TryFrom<&str> for Attribute {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "AlwaysInline" => Ok(Attribute::AlwaysInline),
            "Cold" => Ok(Attribute::Cold),
            "Hot" => Ok(Attribute::Hot),
            "MinSize" => Ok(Attribute::MinSize),
            "OptimizeForSize" => Ok(Attribute::OptimizeForSize),
            "NoInline" => Ok(Attribute::NoInline),
            "WillReturn" => Ok(Attribute::WillReturn),
            "NoReturn" => Ok(Attribute::NoReturn),
            "MustProgress" => Ok(Attribute::MustProgress),
            _ => Err(value.to_owned()),
        }
    }
}
