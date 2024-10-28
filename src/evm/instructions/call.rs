//!
//! Translates a contract call.
//!

use inkwell::values::BasicValue;

use crate::context::pointer::Pointer;
use crate::context::value::Value;
use crate::context::IContext;
use crate::evm::context::address_space::AddressSpace;
use crate::evm::context::Context;
use crate::evm::Dependency;

///
/// Translates an external call.
///
#[allow(clippy::too_many_arguments)]
pub fn call<'ctx, D>(
    context: &mut Context<'ctx, D>,
    gas: inkwell::values::IntValue<'ctx>,
    address: inkwell::values::IntValue<'ctx>,
    value: inkwell::values::IntValue<'ctx>,
    input_offset: inkwell::values::IntValue<'ctx>,
    input_length: inkwell::values::IntValue<'ctx>,
    output_offset: inkwell::values::IntValue<'ctx>,
    output_length: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency,
{
    let input_offset_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        input_offset,
        "call_input_offset_pointer",
    )?;
    let output_offset_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        output_offset,
        "call_output_offset_pointer",
    )?;

    Ok(context
        .build_call(
            context.intrinsics().call,
            &[
                gas.as_basic_value_enum(),
                address.as_basic_value_enum(),
                value.as_basic_value_enum(),
                input_offset_pointer.as_basic_value_enum(),
                input_length.as_basic_value_enum(),
                output_offset_pointer.as_basic_value_enum(),
                output_length.as_basic_value_enum(),
            ],
            "call",
        )?
        .expect("Always exists"))
}

///
/// Translates a static call.
///
#[allow(clippy::too_many_arguments)]
pub fn static_call<'ctx, D>(
    context: &mut Context<'ctx, D>,
    gas: inkwell::values::IntValue<'ctx>,
    address: inkwell::values::IntValue<'ctx>,
    input_offset: inkwell::values::IntValue<'ctx>,
    input_length: inkwell::values::IntValue<'ctx>,
    output_offset: inkwell::values::IntValue<'ctx>,
    output_length: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency,
{
    let input_offset_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        input_offset,
        "staticcall_input_offset_pointer",
    )?;
    let output_offset_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        output_offset,
        "staticcall_output_offset_pointer",
    )?;

    Ok(context
        .build_call(
            context.intrinsics().staticcall,
            &[
                gas.as_basic_value_enum(),
                address.as_basic_value_enum(),
                input_offset_pointer.as_basic_value_enum(),
                input_length.as_basic_value_enum(),
                output_offset_pointer.as_basic_value_enum(),
                output_length.as_basic_value_enum(),
            ],
            "static_call",
        )?
        .expect("Always exists"))
}

///
/// Translates a delegate call.
///
#[allow(clippy::too_many_arguments)]
pub fn delegate_call<'ctx, D>(
    context: &mut Context<'ctx, D>,
    gas: inkwell::values::IntValue<'ctx>,
    address: inkwell::values::IntValue<'ctx>,
    input_offset: inkwell::values::IntValue<'ctx>,
    input_length: inkwell::values::IntValue<'ctx>,
    output_offset: inkwell::values::IntValue<'ctx>,
    output_length: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency,
{
    let input_offset_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        input_offset,
        "delegatecall_input_offset_pointer",
    )?;
    let output_offset_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        output_offset,
        "delegatecall_output_offset_pointer",
    )?;

    Ok(context
        .build_call(
            context.intrinsics().delegatecall,
            &[
                gas.as_basic_value_enum(),
                address.as_basic_value_enum(),
                input_offset_pointer.as_basic_value_enum(),
                input_length.as_basic_value_enum(),
                output_offset_pointer.as_basic_value_enum(),
                output_length.as_basic_value_enum(),
            ],
            "delegate_call",
        )?
        .expect("Always exists"))
}

///
/// Translates the Yul `linkersymbol` instruction.
///
pub fn linker_symbol<'ctx, D>(
    _context: &mut Context<'ctx, D>,
    mut _arguments: [Value<'ctx>; 1],
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency,
{
    unimplemented!()
}
