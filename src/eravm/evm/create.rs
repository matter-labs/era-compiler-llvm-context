//!
//! Translates the contract creation instructions.
//!

use inkwell::values::BasicValue;
use num::Zero;

use crate::context::value::Value;
use crate::context::IContext;
use crate::eravm::context::address_space::AddressSpace;
use crate::eravm::context::function::runtime::Runtime;
use crate::eravm::context::Context;
use crate::eravm::Dependency;

///
/// Translates the contract `create` instruction.
///
/// The instruction is simulated by a call to a system contract.
///
pub fn create<'ctx, D>(
    context: &mut Context<'ctx, D>,
    address_space: AddressSpace,
    value: inkwell::values::IntValue<'ctx>,
    input_offset: inkwell::values::IntValue<'ctx>,
    input_length: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency,
{
    let signature_hash =
        era_compiler_common::Hash::keccak256(crate::eravm::DEPLOYER_SIGNATURE_CREATE.as_bytes());
    let signature_hash_value = context.field_const_str_hex(signature_hash.to_string().as_str());

    let salt = context.field_const(0);

    let function = Runtime::deployer_call(context, address_space);
    let result = context
        .build_call(
            function,
            &[
                value.as_basic_value_enum(),
                input_offset.as_basic_value_enum(),
                input_length.as_basic_value_enum(),
                signature_hash_value.as_basic_value_enum(),
                salt.as_basic_value_enum(),
            ],
            "create_deployer_call",
        )?
        .expect("Always exists");

    Ok(result)
}

///
/// Translates the contract `create2` instruction.
///
/// The instruction is simulated by a call to a system contract.
///
pub fn create2<'ctx, D>(
    context: &mut Context<'ctx, D>,
    address_space: AddressSpace,
    value: inkwell::values::IntValue<'ctx>,
    input_offset: inkwell::values::IntValue<'ctx>,
    input_length: inkwell::values::IntValue<'ctx>,
    salt: Option<inkwell::values::IntValue<'ctx>>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency,
{
    let signature_hash =
        era_compiler_common::Hash::keccak256(crate::eravm::DEPLOYER_SIGNATURE_CREATE2.as_bytes());
    let signature_hash_value = context.field_const_str_hex(signature_hash.to_string().as_str());

    let salt = salt.unwrap_or_else(|| context.field_const(0));

    let function = Runtime::deployer_call(context, address_space);
    let result = context
        .build_call(
            function,
            &[
                value.as_basic_value_enum(),
                input_offset.as_basic_value_enum(),
                input_length.as_basic_value_enum(),
                signature_hash_value.as_basic_value_enum(),
                salt.as_basic_value_enum(),
            ],
            "create2_deployer_call",
        )?
        .expect("Always exists");

    Ok(result)
}

///
/// Translates the contract hash instruction, which is actually used to set the hash of the contract
/// being created, or other related auxiliary data.
///
/// Represents `dataoffset` in Yul and `PUSH [$]` in the EVM legacy assembly.
///
pub fn contract_hash<'ctx, D>(
    context: &mut Context<'ctx, D>,
    identifier: String,
) -> anyhow::Result<Value<'ctx>>
where
    D: Dependency,
{
    let code_segment = context
        .code_segment()
        .expect("Contract code segment type is undefined");

    let parent = context.module().get_name().to_str().expect("Always valid");

    let contract_path =
        context
            .resolve_path(identifier.as_str())
            .map_err(|error| match code_segment {
                era_compiler_common::CodeSegment::Runtime if identifier.ends_with("_deployed") => {
                    anyhow::anyhow!("type({}).runtimeCode is not supported", identifier)
                }
                _ => error,
            })?;
    if contract_path.as_str() == parent {
        return Ok(Value::new_with_constant(
            context.field_const(0).as_basic_value_enum(),
            num::BigUint::zero(),
        ));
    } else if identifier.ends_with("_deployed")
        && code_segment == era_compiler_common::CodeSegment::Runtime
    {
        anyhow::bail!("type({identifier}).runtimeCode is not supported");
    }

    let hash_string = context.get_dependency_data(identifier.as_str())?;
    let hash_value = context
        .field_const_str_hex(hash_string.as_str())
        .as_basic_value_enum();
    Ok(Value::new_with_original(hash_value, hash_string))
}

///
/// Translates the deployer call header size instruction, Usually, the header consists of:
/// - the deployer contract method signature
/// - the salt if the call is `create2`, or zero if the call is `create1`
/// - the hash of the bytecode of the contract whose instance is being created
/// - the offset of the constructor arguments
/// - the length of the constructor arguments
///
/// If the call is `create1`, the space for the salt is still allocated, because the memory for the
/// header is allocated by the Yul or EVM legacy assembly before it is known which version of
/// `create` is going to be used.
///
/// Represents `datasize` in Yul and `PUSH #[$]` in the EVM legacy assembly.
///
pub fn header_size<'ctx, D>(
    context: &mut Context<'ctx, D>,
    identifier: String,
) -> anyhow::Result<Value<'ctx>>
where
    D: Dependency,
{
    let code_segment = context
        .code_segment()
        .expect("Contract code segment type is undefined");

    let parent = context.module().get_name().to_str().expect("Always valid");

    let contract_path =
        context
            .resolve_path(identifier.as_str())
            .map_err(|error| match code_segment {
                era_compiler_common::CodeSegment::Runtime if identifier.ends_with("_deployed") => {
                    anyhow::anyhow!("type({}).runtimeCode is not supported", identifier)
                }
                _ => error,
            })?;
    if contract_path.as_str() == parent {
        return Ok(Value::new_with_constant(
            context.field_const(0).as_basic_value_enum(),
            num::BigUint::zero(),
        ));
    } else if identifier.ends_with("_deployed")
        && code_segment == era_compiler_common::CodeSegment::Runtime
    {
        anyhow::bail!("type({identifier}).runtimeCode is not supported");
    }

    let size_bigint = num::BigUint::from(crate::eravm::DEPLOYER_CALL_HEADER_SIZE);
    let size_value = context
        .field_const(crate::eravm::DEPLOYER_CALL_HEADER_SIZE as u64)
        .as_basic_value_enum();
    Ok(Value::new_with_constant(size_value, size_bigint))
}
