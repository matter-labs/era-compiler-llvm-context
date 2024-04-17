//!
//! Translates the storage operations.
//!

use crate::context::pointer::Pointer;
use crate::context::IContext;
use crate::eravm::context::address_space::AddressSpace;
use crate::eravm::context::Context;
use crate::eravm::Dependency;

///
/// Translates the storage load.
///
pub fn load<'ctx, D>(
    context: &mut Context<'ctx, D>,
    position: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    let position_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Storage,
        context.field_type(),
        position,
        "storage_load_position_pointer",
    )?;
    let value = context.build_load(position_pointer, "storage_load_value")?;
    Ok(value)
}

///
/// Translates the storage store.
///
pub fn store<'ctx, D>(
    context: &mut Context<'ctx, D>,
    position: inkwell::values::IntValue<'ctx>,
    value: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()>
where
    D: Dependency + Clone,
{
    let position_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Storage,
        context.field_type(),
        position,
        "storage_store_position_pointer",
    )?;
    context.build_store(position_pointer, value)?;
    Ok(())
}

///
/// Translates the transient storage load.
///
pub fn transient_load<'ctx, D>(
    context: &mut Context<'ctx, D>,
    position: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    let position_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::TransientStorage,
        context.field_type(),
        position,
        "transient_storage_load_position_pointer",
    )?;
    let value = context.build_load(position_pointer, "transient_storage_load_value")?;
    Ok(value)
}

///
/// Translates the transient storage store.
///
pub fn transient_store<'ctx, D>(
    context: &mut Context<'ctx, D>,
    position: inkwell::values::IntValue<'ctx>,
    value: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()>
where
    D: Dependency + Clone,
{
    let position_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::TransientStorage,
        context.field_type(),
        position,
        "transient_storage_store_position_pointer",
    )?;
    context.build_store(position_pointer, value)?;
    Ok(())
}
