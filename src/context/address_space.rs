//!
//! The LLVM IR address space trait.
//!

///
/// The LLVM IR address space trait.
///
pub trait IAddressSpace {
    ///
    /// Returns the stack address space.
    ///
    fn stack() -> Self;

    ///
    /// Returns the heap address space.
    ///
    fn heap() -> Self;

    ///
    /// Returns the auxiliary heap address space.
    ///
    fn aux_heap() -> Self;

    ///
    /// Returns the calldata address space.
    ///
    fn calldata() -> Self;

    ///
    /// Returns the return data address space.
    ///
    fn return_data() -> Self;

    ///
    /// Returns the generic address space.
    ///
    fn generic() -> Self;

    ///
    /// Returns the code address space.
    ///
    fn code() -> Self;

    ///
    /// Returns the storage address space.
    ///
    fn storage() -> Self;

    ///
    /// Returns the transient storage address space.
    ///
    fn transient_storage() -> Self;
}
