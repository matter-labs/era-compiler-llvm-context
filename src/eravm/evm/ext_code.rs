//!
//! Translates the external code operations.
//!

use crate::context::IContext;
use crate::eravm::context::Context;

///
/// Translates the `extcodesize` instruction.
///
pub fn size<'ctx>(
    context: &mut Context<'ctx>,
    address: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
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
pub fn hash<'ctx>(
    context: &mut Context<'ctx>,
    address: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    crate::eravm::evm::call::request(
        context,
        context.field_const(zkevm_opcode_defs::ADDRESS_ACCOUNT_CODE_STORAGE.into()),
        "getCodeHash(uint256)",
        vec![address],
    )
}
