//!
//! The address space aliases.
//!

use crate::context::address_space::IAddressSpace;

///
/// The address space aliases.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AddressSpace {
    /// The stack memory.
    Stack,
    /// The heap memory.
    Heap,
    /// The auxiliary heap memory.
    HeapAuxiliary,
    /// The generic memory page.
    Generic,
    /// The code area.
    Code,
    /// The storage.
    Storage,
    /// The transient storage.
    TransientStorage,
}

impl IAddressSpace for AddressSpace {
    fn stack() -> Self {
        Self::Stack
    }

    fn heap() -> Self {
        Self::Heap
    }

    fn aux_heap() -> Self {
        Self::HeapAuxiliary
    }

    fn calldata() -> Self {
        panic!("Only available for EVM");
    }

    fn return_data() -> Self {
        panic!("Only available for EVM");
    }

    fn generic() -> Self {
        Self::Generic
    }

    fn code() -> Self {
        Self::Code
    }

    fn storage() -> Self {
        Self::Storage
    }

    fn transient_storage() -> Self {
        Self::TransientStorage
    }
}

impl From<AddressSpace> for inkwell::AddressSpace {
    fn from(value: AddressSpace) -> Self {
        match value {
            AddressSpace::Stack => Self::from(0),
            AddressSpace::Heap => Self::from(1),
            AddressSpace::HeapAuxiliary => Self::from(2),
            AddressSpace::Generic => Self::from(3),
            AddressSpace::Code => Self::from(4),
            AddressSpace::Storage => Self::from(5),
            AddressSpace::TransientStorage => Self::from(6),
        }
    }
}
