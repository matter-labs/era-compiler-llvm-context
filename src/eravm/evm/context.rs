//!
//! Translates the context getter instructions.
//!

use inkwell::values::BasicValue;

use crate::context::IContext;
use crate::eravm::context::Context;

///
/// Translates the `gas_limit` instruction.
///
pub fn gas_limit<'ctx>(
    context: &mut Context<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    crate::eravm::evm::call::request(
        context,
        context.field_const(zkevm_opcode_defs::ADDRESS_SYSTEM_CONTEXT.into()),
        "blockGasLimit()",
        vec![],
    )
}

///
/// Translates the `gas_price` instruction.
///
pub fn gas_price<'ctx>(
    context: &mut Context<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    crate::eravm::evm::call::request(
        context,
        context.field_const(zkevm_opcode_defs::ADDRESS_SYSTEM_CONTEXT.into()),
        "gasPrice()",
        vec![],
    )
}

///
/// Translates the `tx.origin` instruction.
///
pub fn origin<'ctx>(
    context: &mut Context<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    crate::eravm::evm::call::request(
        context,
        context.field_const(zkevm_opcode_defs::ADDRESS_SYSTEM_CONTEXT.into()),
        "origin()",
        vec![],
    )
}

///
/// Translates the `chain_id` instruction.
///
pub fn chain_id<'ctx>(
    context: &mut Context<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    crate::eravm::evm::call::request(
        context,
        context.field_const(zkevm_opcode_defs::ADDRESS_SYSTEM_CONTEXT.into()),
        "chainId()",
        vec![],
    )
}

///
/// Translates the `block_number` instruction.
///
pub fn block_number<'ctx>(
    context: &mut Context<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    crate::eravm::evm::call::request(
        context,
        context.field_const(zkevm_opcode_defs::ADDRESS_SYSTEM_CONTEXT.into()),
        "getBlockNumber()",
        vec![],
    )
}

///
/// Translates the `block_timestamp` instruction.
///
pub fn block_timestamp<'ctx>(
    context: &mut Context<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    crate::eravm::evm::call::request(
        context,
        context.field_const(zkevm_opcode_defs::ADDRESS_SYSTEM_CONTEXT.into()),
        "getBlockTimestamp()",
        vec![],
    )
}

///
/// Translates the `block_hash` instruction.
///
pub fn block_hash<'ctx>(
    context: &mut Context<'ctx>,
    index: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    crate::eravm::evm::call::request(
        context,
        context.field_const(zkevm_opcode_defs::ADDRESS_SYSTEM_CONTEXT.into()),
        "getBlockHashEVM(uint256)",
        vec![index],
    )
}

///
/// Translates the `difficulty` instruction.
///
pub fn difficulty<'ctx>(
    context: &mut Context<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    crate::eravm::evm::call::request(
        context,
        context.field_const(zkevm_opcode_defs::ADDRESS_SYSTEM_CONTEXT.into()),
        "difficulty()",
        vec![],
    )
}

///
/// Translates the `coinbase` instruction.
///
pub fn coinbase<'ctx>(
    context: &mut Context<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    crate::eravm::evm::call::request(
        context,
        context.field_const(zkevm_opcode_defs::ADDRESS_SYSTEM_CONTEXT.into()),
        "coinbase()",
        vec![],
    )
}

///
/// Translates the `basefee` instruction.
///
pub fn basefee<'ctx>(
    context: &mut Context<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    crate::eravm::evm::call::request(
        context,
        context.field_const(zkevm_opcode_defs::ADDRESS_SYSTEM_CONTEXT.into()),
        "baseFee()",
        vec![],
    )
}

///
/// Translates the `msize` instruction.
///
pub fn msize<'ctx>(
    context: &mut Context<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    let meta = context
        .build_call(context.intrinsics().meta, &[], "msize_meta")?
        .expect("Always exists");
    let meta_shifted = context.builder().build_right_shift(
        meta.into_int_value(),
        context.field_const(era_compiler_common::BIT_LENGTH_X64 as u64),
        false,
        "msize_meta_shifted",
    )?;
    let result =
        context
            .builder()
            .build_and(meta_shifted, context.field_const(u32::MAX as u64), "msize")?;
    Ok(result.as_basic_value_enum())
}
