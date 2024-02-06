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
    /// The calldata memory.
    Calldata,
    /// The return data memory.
    ReturnData,
    /// The code memory.
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
        panic!("Only available for zkVM")
    }

    fn calldata() -> Self {
        Self::Calldata
    }

    fn return_data() -> Self {
        Self::ReturnData
    }

    fn generic() -> Self {
        panic!("Only available for zkVM");
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
            AddressSpace::Calldata => Self::from(2),
            AddressSpace::ReturnData => Self::from(3),
            AddressSpace::Code => Self::from(4),
            AddressSpace::Storage => Self::from(5),
            AddressSpace::TransientStorage => Self::from(6),
        }
    }
}
