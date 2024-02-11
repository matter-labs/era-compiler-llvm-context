//!
//! Translates the heap memory operations.
//!

use inkwell::values::BasicValue;

use crate::context::pointer::Pointer;
use crate::context::IContext;
use crate::evm::context::address_space::AddressSpace;
use crate::evm::context::Context;
use crate::evm::Dependency;

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
        context.byte_type(),
        offset,
        "store_byte_pointer",
    );

    context.build_call(
        context.intrinsics().mstore8,
        &[pointer.as_basic_value_enum(), value.as_basic_value_enum()],
        "mstore8",
    );
    Ok(())
}
