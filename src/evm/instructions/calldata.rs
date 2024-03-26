//!
//! Translates the calldata instructions.
//!

use crate::context::pointer::Pointer;
use crate::context::IContext;
use crate::evm::context::address_space::AddressSpace;
use crate::evm::context::Context;
use crate::evm::Dependency;

///
/// Translates the calldata load.
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
        AddressSpace::Calldata,
        context.field_type(),
        offset,
        "calldataload_pointer",
    )?;
    let result = context.build_load(pointer, "calldata_load_result")?;
    Ok(result)
}

///
/// Translates the calldata size.
///
pub fn size<'ctx, D>(
    context: &mut Context<'ctx, D>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    Ok(context
        .build_call(context.intrinsics().calldatasize, &[], "calldatasize")?
        .expect("Always exists"))
}

///
/// Translates the calldata copy.
///
pub fn copy<'ctx, D>(
    context: &mut Context<'ctx, D>,
    destination_offset: inkwell::values::IntValue<'ctx>,
    source_offset: inkwell::values::IntValue<'ctx>,
    size: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()>
where
    D: Dependency + Clone,
{
    let destination = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        destination_offset,
        "calldatacopy_destination_pointer",
    )?;

    let source = Pointer::new_with_offset(
        context,
        AddressSpace::Calldata,
        context.byte_type(),
        source_offset,
        "calldatacopy_source_pointer",
    )?;

    context.build_memcpy(
        context.intrinsics().memory_copy_from_calldata,
        destination,
        source,
        size,
        "calldatacopy_memcpy",
    )?;
    Ok(())
}
