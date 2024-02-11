//!
//! Translates the value and balance operations.
//!

use inkwell::values::BasicValue;

use crate::context::IContext;
use crate::evm::context::Context;
use crate::evm::Dependency;

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
        .build_call(context.intrinsics().gas, &[], "gas")
        .expect("Always exists"))
}

///
/// Translates the `callvalue` instruction.
///
pub fn callvalue<'ctx, D>(
    context: &mut Context<'ctx, D>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    Ok(context
        .build_call(context.intrinsics().callvalue, &[], "callvalue")
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
    Ok(context
        .build_call(
            context.intrinsics().balance,
            &[address.as_basic_value_enum()],
            "balance",
        )
        .expect("Always exists"))
}

///
/// Translates the `selfbalance` instructions.
///
pub fn self_balance<'ctx, D>(
    context: &mut Context<'ctx, D>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    Ok(context
        .build_call(context.intrinsics().selfbalance, &[], "selfbalance")
        .expect("Always exists"))
}

///
/// Translates the `selfdestruct` instructions.
///
pub fn self_destruct<'ctx, D>(
    context: &mut Context<'ctx, D>,
    address: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    Ok(context
        .build_call(
            context.intrinsics().selfdestruct,
            &[address.as_basic_value_enum()],
            "selfdestruct",
        )
        .expect("Always exists"))
}
