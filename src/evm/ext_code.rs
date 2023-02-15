//!
//! Translates the external code operations.
//!

use crate::context::Context;
use crate::Dependency;

///
/// Translates the `extcodesize` instruction.
///
pub fn size<'ctx, D>(
    context: &mut Context<'ctx, D>,
    address: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency,
{
    crate::evm::call::request(
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
    D: Dependency,
{
    crate::evm::call::request(
        context,
        context.field_const(zkevm_opcode_defs::ADDRESS_ACCOUNT_CODE_STORAGE.into()),
        "getCodeHash(uint256)",
        vec![address],
    )
}
