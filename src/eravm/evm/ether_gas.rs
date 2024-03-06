//!
//! Translates the value and balance operations.
//!

use crate::context::IContext;
use crate::eravm::context::Context;
use crate::eravm::Dependency;

///
/// Translates the `gas` instruction.
///
pub fn gas<'ctx, D>(
    context: &mut Context<'ctx, D>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    Ok(context
        .build_call(context.intrinsics().gas_left, &[], "gas_left")
        .expect("Always exists"))
}

///
/// Translates the `value` instruction.
///
pub fn value<'ctx, D>(
    context: &mut Context<'ctx, D>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    Ok(context
        .build_call(context.intrinsics().get_u128, &[], "get_u128_value")
        .expect("Always exists"))
}

///
/// Translates the `balance` instructions.
///
pub fn balance<'ctx, D>(
    context: &mut Context<'ctx, D>,
    address: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    crate::eravm::evm::call::request(
        context,
        context.field_const(zkevm_opcode_defs::ADDRESS_ETH_TOKEN.into()),
        "balanceOf(uint256)",
        vec![address],
    )
}
