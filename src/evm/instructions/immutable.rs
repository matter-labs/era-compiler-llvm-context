//!
//! Translates the contract immutable operations.
//!

use crate::evm::context::Context;

///
/// Translates the contract immutable load.
///
pub fn load<'ctx>(
    _context: &mut Context<'ctx>,
    _index: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    todo!()
}

///
/// Translates the contract immutable store.
///
pub fn store<'ctx>(
    _context: &mut Context<'ctx>,
    _index: inkwell::values::IntValue<'ctx>,
    _value: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()> {
    todo!()
}
