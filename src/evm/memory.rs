//!
//! Translates the heap memory operations.
//!

use crate::context::address_space::AddressSpace;
use crate::context::pointer::Pointer;
use crate::context::Context;
use crate::Dependency;

///
/// Translates the `mload` instruction.
///
/// Uses the main heap.
///
pub fn load<'ctx, D>(
    context: &mut Context<'ctx, D>,
    offset: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    let pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.field_type(),
        offset,
        "memory_load_pointer",
    );
    let result = context.build_load(pointer, "memory_load_result");
    Ok(result)
}

///
/// Translates the `mstore` instruction.
///
/// Uses the main heap.
///
pub fn store<'ctx, D>(
    context: &mut Context<'ctx, D>,
    offset: inkwell::values::IntValue<'ctx>,
    value: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()>
where
    D: Dependency + Clone,
{
    let pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.field_type(),
        offset,
        "memory_store_pointer",
    );
    context.build_store(pointer, value);
    Ok(())
}

///
/// Translates the `mstore8` instruction.
///
/// Uses the main heap.
///
pub fn store_byte<'ctx, D>(
    context: &mut Context<'ctx, D>,
    offset: inkwell::values::IntValue<'ctx>,
    value: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()>
where
    D: Dependency + Clone,
{
    let pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.field_type(),
        offset,
        "memory_store_byte_original_value_pointer",
    );

    let original_value = context
        .build_load(pointer, "memory_store_byte_original_value")
        .into_int_value();
    let original_value_shifted_left = context.builder().build_left_shift(
        original_value,
        context.field_const(compiler_common::BIT_LENGTH_BYTE as u64),
        "memory_store_byte_original_value_shifted_left",
    );
    let original_value_shifted_right = context.builder().build_right_shift(
        original_value_shifted_left,
        context.field_const(compiler_common::BIT_LENGTH_BYTE as u64),
        false,
        "memory_store_byte_original_value_shifted_right",
    );

    let value_shifted = context.builder().build_left_shift(
        value,
        context.field_const(
            ((compiler_common::BYTE_LENGTH_FIELD - 1) * compiler_common::BIT_LENGTH_BYTE) as u64,
        ),
        "memory_store_byte_value_shifted",
    );
    let result = context.builder().build_or(
        original_value_shifted_right,
        value_shifted,
        "memory_store_byte_result",
    );

    context.build_store(pointer, result);
    Ok(())
}
