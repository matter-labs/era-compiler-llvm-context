//!
//! The LLVM pointer.
//!

use inkwell::types::BasicType;
use inkwell::values::BasicValue;

use crate::context::IContext;
use crate::evm::context::address_space::AddressSpace;
use crate::evm::context::Context;
use crate::evm::Dependency;

///
/// The LLVM pointer.
///
#[derive(Debug, Clone, Copy)]
pub struct Pointer<'ctx> {
    /// The pointee type.
    pub r#type: inkwell::types::BasicTypeEnum<'ctx>,
    /// The address space.
    pub address_space: AddressSpace,
    /// The pointer value.
    pub value: inkwell::values::PointerValue<'ctx>,
}

impl<'ctx> Pointer<'ctx> {
    ///
    /// A shortcut constructor.
    ///
    pub fn new<T>(
        r#type: T,
        address_space: AddressSpace,
        value: inkwell::values::PointerValue<'ctx>,
    ) -> Self
    where
        T: BasicType<'ctx>,
    {
        Self {
            r#type: r#type.as_basic_type_enum(),
            address_space,
            value,
        }
    }

    ///
    /// Wraps a 256-bit primitive type pointer.
    ///
    pub fn new_stack_field<D>(
        context: &Context<'ctx, D>,
        value: inkwell::values::PointerValue<'ctx>,
    ) -> Self
    where
        D: Dependency + Clone,
    {
        Self {
            r#type: context.field_type().as_basic_type_enum(),
            address_space: AddressSpace::Stack,
            value,
        }
    }

    ///
    /// Creates a new pointer with the specified `offset`.
    ///
    pub fn new_with_offset<D, T>(
        context: &Context<'ctx, D>,
        address_space: AddressSpace,
        r#type: T,
        offset: inkwell::values::IntValue<'ctx>,
        name: &str,
    ) -> Self
    where
        D: Dependency + Clone,
        T: BasicType<'ctx>,
    {
        assert_ne!(
            address_space,
            AddressSpace::Stack,
            "Stack pointers cannot be addressed"
        );

        let value = context.builder.build_int_to_ptr(
            offset,
            context.byte_type().ptr_type(address_space.into()),
            name,
        );
        Self::new(r#type, address_space, value)
    }

    ///
    /// Casts the pointer into another type.
    ///
    pub fn cast<T>(self, r#type: T) -> Self
    where
        T: BasicType<'ctx>,
    {
        Self {
            r#type: r#type.as_basic_type_enum(),
            address_space: self.address_space,
            value: self.value,
        }
    }

    ///
    /// Converts the pointer to a value enum.
    ///
    pub fn as_basic_value_enum(self) -> inkwell::values::BasicValueEnum<'ctx> {
        self.value.as_basic_value_enum()
    }
}
