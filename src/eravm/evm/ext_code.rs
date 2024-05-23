//!
//! Translates the external code operations.
//!

use inkwell::types::BasicType;

use crate::context::pointer::Pointer;
use crate::context::IContext;
use crate::eravm::context::address_space::AddressSpace;
use crate::eravm::context::Context;
use crate::eravm::Dependency;

///
/// Translates the `extcodesize` instruction.
///
pub fn size<'ctx, D>(
    context: &mut Context<'ctx, D>,
    address: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    crate::eravm::evm::call::request(
        context,
        context.field_const(zkevm_opcode_defs::ADDRESS_ACCOUNT_CODE_STORAGE.into()),
        "getCodeSize(uint256)",
        vec![address],
    )
}

///
/// Translates the `extcodehash` instruction.
///
pub fn hash<'ctx, D>(
    context: &mut Context<'ctx, D>,
    address: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    crate::eravm::evm::call::request(
        context,
        context.field_const(zkevm_opcode_defs::ADDRESS_ACCOUNT_CODE_STORAGE.into()),
        "getCodeHash(uint256)",
        vec![address],
    )
}

///
/// Translates the `extcodecopy` instruction.
///
pub fn copy<'ctx, D>(
    context: &mut Context<'ctx, D>,
    address: inkwell::values::IntValue<'ctx>,
    destination_offset: inkwell::values::IntValue<'ctx>,
    source_offset: inkwell::values::IntValue<'ctx>,
    size: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()>
where
    D: Dependency + Clone,
{
    let hash = hash(context, address)?;

    let source_pointer = crate::eravm::evm::call::request_fallback(
        context,
        context.field_const(zkevm_opcode_defs::ADDRESS_CODE_ORACLE.into()),
        vec![hash.into_int_value()],
    )?;
    let source = context.build_gep(
        Pointer::<AddressSpace>::new(
            context.byte_type(),
            AddressSpace::Generic,
            source_pointer.into_pointer_value(),
        ),
        &[source_offset],
        context.byte_type().as_basic_type_enum(),
        "extcodecopy_source_pointer",
    )?;

    let destination = Pointer::<AddressSpace>::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        destination_offset,
        "extcodecopy_destination_pointer",
    )?;

    context.build_memcpy(
        context.intrinsics().memory_copy_from_generic,
        destination,
        source,
        size,
        "extcodecopy_memcpy_from_return_data",
    )?;
    Ok(())
}
