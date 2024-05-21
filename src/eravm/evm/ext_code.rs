//!
//! Translates the external code operations.
//!

use crate::context::IContext;
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
    crate::eravm::evm::call::request_fallback(
        context,
        context.field_const(zkevm_opcode_defs::ADDRESS_CODE_ORACLE.into()),
        vec![hash.into_int_value()],
    )?;
    crate::eravm::evm::return_data::copy(context, destination_offset, source_offset, size)?;
    Ok(())
}
