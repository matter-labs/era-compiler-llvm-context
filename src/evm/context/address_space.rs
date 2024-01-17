//!
//! The address space aliases.
//!

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
        }
    }
}
