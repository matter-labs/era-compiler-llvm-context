//!
//! Translates the general instructions of the EraVM Yul extension.
//!

use inkwell::values::BasicValue;

use crate::context::IContext;
use crate::eravm::context::address_space::AddressSpace;
use crate::eravm::context::Context;
use crate::eravm::Dependency;

///
/// Generates a call to L1.
///
pub fn to_l1<'ctx, D>(
    context: &mut Context<'ctx, D>,
    is_first: inkwell::values::IntValue<'ctx>,
    in_0: inkwell::values::IntValue<'ctx>,
    in_1: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    let join_block = context.append_basic_block("contract_call_toL1_join_block");

    let contract_call_tol1_is_first_block =
        context.append_basic_block("contract_call_toL1_is_first_block");
    let contract_call_tol1_is_not_first_block =
        context.append_basic_block("contract_call_toL1_is_not_first_block");

    let is_first_equals_zero = context.builder().build_int_compare(
        inkwell::IntPredicate::EQ,
        is_first,
        context.field_const(0),
        "contract_call_toL1_is_first_equals_zero",
    )?;
    context.build_conditional_branch(
        is_first_equals_zero,
        contract_call_tol1_is_not_first_block,
        contract_call_tol1_is_first_block,
    )?;

    {
        context.set_basic_block(contract_call_tol1_is_not_first_block);
        let is_first = context.field_const(0);
        context.build_call(
            context.intrinsics().to_l1,
            &[
                in_0.as_basic_value_enum(),
                in_1.as_basic_value_enum(),
                is_first.as_basic_value_enum(),
            ],
            "contract_call_simulation_tol1",
        )?;
        context.build_unconditional_branch(join_block)?;
    }

    {
        context.set_basic_block(contract_call_tol1_is_first_block);
        let is_first = context.field_const(1);
        context.build_call(
            context.intrinsics().to_l1,
            &[
                in_0.as_basic_value_enum(),
                in_1.as_basic_value_enum(),
                is_first.as_basic_value_enum(),
            ],
            "contract_call_simulation_tol1",
        )?;
        context.build_unconditional_branch(join_block)?;
    }

    context.set_basic_block(join_block);
    Ok(context.field_const(1).as_basic_value_enum())
}

///
/// Generates a `code source` call.
///
pub fn code_source<'ctx, D>(
    context: &mut Context<'ctx, D>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    let result = context
        .build_call(
            context.intrinsics().code_source,
            &[],
            "contract_call_simulation_code_source",
        )?
        .expect("Always exists");
    Ok(result)
}

///
/// Generates a precompile call.
///
pub fn precompile<'ctx, D>(
    context: &mut Context<'ctx, D>,
    in_0: inkwell::values::IntValue<'ctx>,
    gas_left: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    let result = context
        .build_call(
            context.intrinsics().precompile,
            &[in_0.as_basic_value_enum(), gas_left.as_basic_value_enum()],
            "contract_call_simulation_precompile",
        )?
        .expect("Always exists");
    Ok(result)
}

///
/// Generates a decommit call.
///
pub fn decommit<'ctx, D>(
    context: &mut Context<'ctx, D>,
    in_0: inkwell::values::IntValue<'ctx>,
    gas_left: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    let result = context
        .build_call(
            context.intrinsics().decommit,
            &[in_0.as_basic_value_enum(), gas_left.as_basic_value_enum()],
            "contract_call_simulation_decommit",
        )
        .expect("Always exists");
    context.set_global(
        crate::eravm::GLOBAL_DECOMMIT_POINTER,
        context.byte_type().ptr_type(AddressSpace::Generic.into()),
        AddressSpace::Stack,
        result,
    );
    Ok(result)
}

///
/// Generates a `meta` call.
///
pub fn meta<'ctx, D>(
    context: &mut Context<'ctx, D>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    let result = context
        .build_call(
            context.intrinsics().meta,
            &[],
            "contract_call_simulation_meta",
        )?
        .expect("Always exists");
    Ok(result)
}

///
/// Generates a `u128` context value setter call.
///
pub fn set_context_value<'ctx, D>(
    context: &mut Context<'ctx, D>,
    value: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    context.build_call(
        context.intrinsics().set_u128,
        &[value.as_basic_value_enum()],
        "contract_call_simulation_set_context_value",
    )?;
    Ok(context.field_const(1).as_basic_value_enum())
}

///
/// Generates a public data price setter call.
///
pub fn set_pubdata_price<'ctx, D>(
    context: &mut Context<'ctx, D>,
    value: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    context.build_call(
        context.intrinsics().set_pubdata_price,
        &[value.as_basic_value_enum()],
        "contract_call_simulation_set_pubdata_price",
    )?;
    Ok(context.field_const(1).as_basic_value_enum())
}

///
/// Generates a transaction counter increment call.
///
pub fn increment_tx_counter<'ctx, D>(
    context: &mut Context<'ctx, D>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    context.build_call(
        context.intrinsics().increment_tx_counter,
        &[],
        "contract_call_simulation_increment_tx_counter",
    )?;
    Ok(context.field_const(1).as_basic_value_enum())
}

///
/// Generates an event call.
///
pub fn event<'ctx, D>(
    context: &mut Context<'ctx, D>,
    operand_1: inkwell::values::IntValue<'ctx>,
    operand_2: inkwell::values::IntValue<'ctx>,
    is_initializer: bool,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    context.build_call(
        context.intrinsics().event,
        &[
            operand_1.as_basic_value_enum(),
            operand_2.as_basic_value_enum(),
            context
                .field_const(u64::from(is_initializer))
                .as_basic_value_enum(),
        ],
        if is_initializer {
            "event_initialize"
        } else {
            "event_write"
        },
    )?;
    return Ok(context.field_const(1).as_basic_value_enum());
}
