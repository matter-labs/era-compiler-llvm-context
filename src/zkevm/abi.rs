//!
//! Translates the ABI instructions of the zkEVM Yul extension.
//!

use inkwell::types::BasicType;
use inkwell::values::BasicValue;

use crate::context::address_space::AddressSpace;
use crate::context::pointer::Pointer;
use crate::context::Context;
use crate::Dependency;

///
/// Generates an extra ABI data getter call.
///
pub fn get_extra_abi_data<'ctx, D>(
    context: &mut Context<'ctx, D>,
    index: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    let extra_abi_data_pointer = context.get_global_ptr(crate::GLOBAL_EXTRA_ABI_DATA)?;
    let extra_abi_data_element_pointer = context.build_gep(
        extra_abi_data_pointer,
        &[context.field_const(0), index],
        context.field_type().as_basic_type_enum(),
        "extra_abi_data_element_pointer",
    );
    let extra_abi_data_element = context.build_load(
        extra_abi_data_element_pointer,
        "extra_abi_data_element_value",
    );
    Ok(extra_abi_data_element)
}

///
/// Loads the calldata pointer to the active pointer.
///
pub fn calldata_ptr_to_active<'ctx, D>(
    context: &mut Context<'ctx, D>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    let calldata_pointer = context.get_global(crate::GLOBAL_CALLDATA_POINTER)?;
    context.set_global(
        crate::GLOBAL_ACTIVE_POINTER,
        context.byte_type().ptr_type(AddressSpace::Generic.into()),
        calldata_pointer,
    );
    Ok(context.field_const(1).as_basic_value_enum())
}

///
/// Loads the return data pointer to the active pointer.
///
pub fn return_data_ptr_to_active<'ctx, D>(
    context: &mut Context<'ctx, D>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    let calldata_pointer = context.get_global(crate::GLOBAL_RETURN_DATA_POINTER)?;
    context.set_global(
        crate::GLOBAL_ACTIVE_POINTER,
        context.byte_type().ptr_type(AddressSpace::Generic.into()),
        calldata_pointer,
    );
    Ok(context.field_const(1).as_basic_value_enum())
}

///
/// Shifts the active pointer by the specified `offset`.
///
pub fn active_ptr_add_assign<'ctx, D>(
    context: &mut Context<'ctx, D>,
    offset: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    let active_pointer = context.get_global(crate::GLOBAL_ACTIVE_POINTER)?;
    let active_pointer_shifted = context.build_gep(
        Pointer::new(
            context.byte_type(),
            AddressSpace::Generic,
            active_pointer.into_pointer_value(),
        ),
        &[offset],
        context.byte_type().as_basic_type_enum(),
        "active_pointer_shifted",
    );
    context.set_global(
        crate::GLOBAL_ACTIVE_POINTER,
        context.byte_type().ptr_type(AddressSpace::Generic.into()),
        active_pointer_shifted.value,
    );
    Ok(context.field_const(1).as_basic_value_enum())
}

///
/// Shrinks the active pointer by the specified `offset`.
///
pub fn active_ptr_shrink_assign<'ctx, D>(
    context: &mut Context<'ctx, D>,
    offset: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    let active_pointer = context.get_global(crate::GLOBAL_ACTIVE_POINTER)?;
    let active_pointer_shrunken = context
        .build_call(
            context.intrinsics().pointer_shrink,
            &[active_pointer, offset.as_basic_value_enum()],
            "active_pointer_shrunken",
        )
        .expect("Always returns a pointer");
    context.set_global(
        crate::GLOBAL_ACTIVE_POINTER,
        context.byte_type().ptr_type(AddressSpace::Generic.into()),
        active_pointer_shrunken,
    );
    Ok(context.field_const(1).as_basic_value_enum())
}

///
/// Writes the specified `data` into the upper 128 bits of the active pointer.
///
pub fn active_ptr_pack_assign<'ctx, D>(
    context: &mut Context<'ctx, D>,
    data: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    let active_pointer = context.get_global(crate::GLOBAL_ACTIVE_POINTER)?;
    let active_pointer_packed = context
        .build_call(
            context.intrinsics().pointer_pack,
            &[active_pointer, data.as_basic_value_enum()],
            "active_pointer_packed",
        )
        .expect("Always returns a pointer");
    context.set_global(
        crate::GLOBAL_ACTIVE_POINTER,
        context.byte_type().ptr_type(AddressSpace::Generic.into()),
        active_pointer_packed,
    );
    Ok(context.field_const(1).as_basic_value_enum())
}
