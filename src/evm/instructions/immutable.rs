//!
//! Translates the contract immutable operations.
//!

use crate::evm::context::Context;
use crate::evm::Dependency;

///
/// Translates the contract immutable load.
///
pub fn load<'ctx, D>(
    _context: &mut Context<'ctx, D>,
    _index: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    todo!()
}

///
/// Translates the contract immutable store.
///
pub fn store<'ctx, D>(
    _context: &mut Context<'ctx, D>,
    _index: inkwell::values::IntValue<'ctx>,
    _value: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()>
where
    D: Dependency + Clone,
{
    todo!()
}
