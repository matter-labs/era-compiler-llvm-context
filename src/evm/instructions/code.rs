//!
//! Translates the external code operations.
//!

use inkwell::values::BasicValue;

use crate::context::pointer::Pointer;
use crate::context::IContext;
use crate::evm::context::address_space::AddressSpace;
use crate::evm::context::Context;
use crate::evm::Dependency;

///
/// Translates the `codesize` instruction.
///
pub fn size<'ctx, D>(
    context: &mut Context<'ctx, D>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    Ok(context
        .build_call(context.intrinsics().codesize, &[], "codesize")?
        .expect("Always exists"))
}

///
/// Translates the `codecopy` instruction.
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
        "codecopy_destination_pointer",
    )?;

    let source = Pointer::new_with_offset(
        context,
        AddressSpace::Code,
        context.byte_type(),
        source_offset,
        "codecopy_source_pointer",
    )?;

    context.build_memcpy(
        context.intrinsics().memory_copy_from_code,
        destination,
        source,
        size,
        "codecopy_memcpy",
    )?;
    Ok(())
}

///
/// Translates the `dataoffset` instruction.
///
pub fn data_offset<'ctx, D>(
    context: &mut Context<'ctx, D>,
    object_name: &str,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    if !object_name.ends_with("_deployed") {
        return Ok(context.field_const(0).as_basic_value_enum());
    }

    let deploy_code_size = context
        .build_call(context.intrinsics().datasize, &[], "deploy_code_size")?
        .expect("Always exists");
    let runtime_code_size = context
        .build_call(
            context.intrinsics().datasize_runtime,
            &[],
            "runtime_code_size",
        )?
        .expect("Always exists");
    let data_offset = context.builder().build_int_sub(
        deploy_code_size.into_int_value(),
        runtime_code_size.into_int_value(),
        "data_offset",
    )?;
    Ok(data_offset.as_basic_value_enum())
}

///
/// Translates the `datasize` instruction.
///
pub fn data_size<'ctx, D>(
    context: &mut Context<'ctx, D>,
    object_name: &str,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    let intrinsic = if object_name.ends_with("_deployed") {
        context.intrinsics().datasize
    } else {
        context.intrinsics().datasize_runtime
    };

    Ok(context
        .build_call(intrinsic, &[], "codesize")?
        .expect("Always exists"))
}

///
/// Translates the `extcodesize` instruction.
///
pub fn ext_size<'ctx, D>(
    context: &mut Context<'ctx, D>,
    address: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    Ok(context
        .build_call(
            context.intrinsics().extcodesize,
            &[address.as_basic_value_enum()],
            "extcodesize",
        )?
        .expect("Always exists"))
}

///
/// Translates the `extcodecopy` instruction.
///
pub fn ext_copy<'ctx, D>(
    context: &mut Context<'ctx, D>,
    address: inkwell::values::IntValue<'ctx>,
    destination_offset: inkwell::values::IntValue<'ctx>,
    source_offset: inkwell::values::IntValue<'ctx>,
    size: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()>
where
    D: Dependency + Clone,
{
    let destination_offset_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        destination_offset,
        "extcodecopy_destination_offset_pointer",
    )?;

    context
        .build_call(
            context.intrinsics().extcodecopy,
            &[
                address.as_basic_value_enum(),
                destination_offset_pointer.as_basic_value_enum(),
                source_offset.as_basic_value_enum(),
                size.as_basic_value_enum(),
            ],
            "extcodecopy",
        )?
        .expect("Always exists");
    Ok(())
}

///
/// Translates the `extcodehash` instruction.
///
pub fn ext_hash<'ctx, D>(
    context: &mut Context<'ctx, D>,
    address: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    Ok(context
        .build_call(
            context.intrinsics().extcodehash,
            &[address.as_basic_value_enum()],
            "extcodehash",
        )?
        .expect("Always exists"))
}
