//!
//! Translates the contract storage operations.
//!

use inkwell::values::BasicValue;

use crate::eravm::context::Context;
use crate::eravm::Dependency;

///
/// Translates the contract storage load.
///
pub fn load<'ctx, D>(
    context: &mut Context<'ctx, D>,
    position: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    let value = context
        .build_call(
            context.intrinsics().storage_load,
            &[position.as_basic_value_enum()],
            "storage_load",
        )
        .expect("Contract storage always returns a value");
    Ok(value)
}

///
/// Translates the contract storage store.
///
/// Beware that the `position` and `value` arguments have different order in Yul and LLVM IR.
///
pub fn store<'ctx, D>(
    context: &mut Context<'ctx, D>,
    position: inkwell::values::IntValue<'ctx>,
    value: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()>
where
    D: Dependency + Clone,
{
    context.build_invoke(
        context.intrinsics().storage_store,
        &[position.as_basic_value_enum(), value.as_basic_value_enum()],
        "storage_store",
    );
    Ok(())
}
