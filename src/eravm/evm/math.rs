//!
//! Translates the mathematics operation.
//!

use inkwell::values::BasicValue;

use crate::eravm::context::function::runtime::Runtime;
use crate::eravm::context::Context;
use crate::eravm::Dependency;

///
/// Translates the `addmod` instruction.
///
/// Implemented as a runtime function in the LLVM back-end.
///
pub fn add_mod<'ctx, D>(
    context: &mut Context<'ctx, D>,
    operand_1: inkwell::values::IntValue<'ctx>,
    operand_2: inkwell::values::IntValue<'ctx>,
    modulo: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    Ok(context
        .build_call(
            context.llvm_runtime().add_mod,
            &[
                operand_1.as_basic_value_enum(),
                operand_2.as_basic_value_enum(),
                modulo.as_basic_value_enum(),
            ],
            "add_mod_call",
        )
        .expect("Always exists"))
}

///
/// Translates the `mulmod` instruction.
///
/// Implemented as a runtime function in the LLVM back-end.
///
pub fn mul_mod<'ctx, D>(
    context: &mut Context<'ctx, D>,
    operand_1: inkwell::values::IntValue<'ctx>,
    operand_2: inkwell::values::IntValue<'ctx>,
    modulo: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    Ok(context
        .build_call(
            context.llvm_runtime().mul_mod,
            &[
                operand_1.as_basic_value_enum(),
                operand_2.as_basic_value_enum(),
                modulo.as_basic_value_enum(),
            ],
            "mul_mod_call",
        )
        .expect("Always exists"))
}

///
/// Translates the `exp` instruction.
///
/// Implemented as the binary exponentiation algorithm.
///
pub fn exponent<'ctx, D>(
    context: &mut Context<'ctx, D>,
    value: inkwell::values::IntValue<'ctx>,
    exponent: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    let function = Runtime::exponent(context);
    Ok(context
        .build_call(
            function,
            &[value.as_basic_value_enum(), exponent.as_basic_value_enum()],
            "exponent_call",
        )
        .expect("Always exists"))
}

///
/// Translates the `signextend` instruction.
///
/// Implemented as a runtime function in the LLVM back-end.
///
pub fn sign_extend<'ctx, D>(
    context: &mut Context<'ctx, D>,
    bytes: inkwell::values::IntValue<'ctx>,
    value: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    let overflow_block = context.append_basic_block("sign_extend_zero");
    let non_overflow_block = context.append_basic_block("sign_extend_non_zero");
    let join_block = context.append_basic_block("sign_extend_join");

    let result_pointer = context.build_alloca(context.field_type(), "sign_extend_result_pointer");
    let condition_is_overflow = context.builder().build_int_compare(
        inkwell::IntPredicate::UGE,
        bytes,
        context.field_const((compiler_common::BYTE_LENGTH_FIELD - 1) as u64),
        "sign_extend_is_overflow",
    );
    context.build_conditional_branch(condition_is_overflow, overflow_block, non_overflow_block);

    context.set_basic_block(overflow_block);
    context.build_store(result_pointer, value);
    context.build_unconditional_branch(join_block);

    context.set_basic_block(non_overflow_block);
    let result = context
        .build_call(
            context.llvm_runtime().sign_extend,
            &[bytes.as_basic_value_enum(), value.as_basic_value_enum()],
            "sign_extend_call",
        )
        .expect("Always exists");
    context.build_store(result_pointer, result);
    context.build_unconditional_branch(join_block);

    context.set_basic_block(join_block);
    let result = context.build_load(result_pointer, "sign_extend_result");
    Ok(result)
}
