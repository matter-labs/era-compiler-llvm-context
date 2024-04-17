//!
//! Translates the contract creation instructions.
//!

use inkwell::values::BasicValue;

use crate::context::pointer::Pointer;
use crate::context::IContext;
use crate::evm::context::address_space::AddressSpace;
use crate::evm::context::Context;
use crate::evm::Dependency;

///
/// Translates the contract `create` instruction.
///
pub fn create<'ctx, D>(
    context: &mut Context<'ctx, D>,
    value: inkwell::values::IntValue<'ctx>,
    input_offset: inkwell::values::IntValue<'ctx>,
    input_length: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    let input_offset_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        input_offset,
        "create_input_offset_pointer",
    )?;

    Ok(context
        .build_call(
            context.intrinsics().create,
            &[
                value.as_basic_value_enum(),
                input_offset_pointer.as_basic_value_enum(),
                input_length.as_basic_value_enum(),
            ],
            "create",
        )?
        .expect("Always exists"))
}

///
/// Translates the contract `create2` instruction.
///
pub fn create2<'ctx, D>(
    context: &mut Context<'ctx, D>,
    value: inkwell::values::IntValue<'ctx>,
    input_offset: inkwell::values::IntValue<'ctx>,
    input_length: inkwell::values::IntValue<'ctx>,
    salt: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    let input_offset_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        input_offset,
        "create2_input_offset_pointer",
    )?;

    Ok(context
        .build_call(
            context.intrinsics().create2,
            &[
                value.as_basic_value_enum(),
                input_offset_pointer.as_basic_value_enum(),
                input_length.as_basic_value_enum(),
                salt.as_basic_value_enum(),
            ],
            "create2",
        )?
        .expect("Always exists"))
}
