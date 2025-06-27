//!
//! Translates the contract immutable operations.
//!

use crate::context::pointer::Pointer;
use crate::context::IContext;
use crate::eravm::context::address_space::AddressSpace;
use crate::eravm::context::Context;

///
/// Translates the contract immutable load.
///
/// In the deploy code the values are read from the auxiliary heap.
/// In the runtime code they are requested from the system contract.
///
pub fn load<'ctx>(
    context: &mut Context<'ctx>,
    index: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    match context.code_segment() {
        None => {
            panic!("Contract code segment is undefined");
        }
        Some(era_compiler_common::CodeSegment::Deploy) => {
            let index_double = context.builder().build_int_mul(
                index,
                context.field_const(2),
                "immutable_load_index_double",
            )?;
            let offset_absolute = context.builder().build_int_add(
                index_double,
                context.field_const(
                    crate::eravm::HEAP_AUX_OFFSET_CONSTRUCTOR_RETURN_DATA
                        + (3 * era_compiler_common::BYTE_LENGTH_FIELD) as u64,
                ),
                "immutable_offset_absolute",
            )?;
            let immutable_pointer = Pointer::new_with_offset(
                context,
                AddressSpace::HeapAuxiliary,
                context.field_type(),
                offset_absolute,
                "immutable_pointer",
            )?;
            let immutable_value = context.build_load(immutable_pointer, "immutable_value")?;
            Ok(immutable_value)
        }
        Some(era_compiler_common::CodeSegment::Runtime) => {
            let code_address = context
                .build_call(
                    context.intrinsics().code_source,
                    &[],
                    "immutable_code_address",
                )?
                .expect("Always exists")
                .into_int_value();
            crate::eravm::evm::call::request(
                context,
                context.field_const(zkevm_opcode_defs::ADDRESS_IMMUTABLE_SIMULATOR.into()),
                "getImmutable(address,uint256)",
                vec![code_address, index],
            )
        }
    }
}

///
/// Translates the contract immutable store.
///
/// In the deploy code the values are written to the auxiliary heap at the predefined offset,
/// being prepared for returning to the system contract for saving.
///
/// Ignored in the runtime code.
///
pub fn store<'ctx>(
    context: &mut Context<'ctx>,
    index: inkwell::values::IntValue<'ctx>,
    value: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()> {
    match context.code_segment() {
        Some(era_compiler_common::CodeSegment::Deploy) => {}
        Some(era_compiler_common::CodeSegment::Runtime) => {
            anyhow::bail!("Setting immutables is only allowed in deploy code");
        }
        None => {
            anyhow::bail!("Code segment is undefined");
        }
    }

    let index_double = context.builder().build_int_mul(
        index,
        context.field_const(2),
        "immutable_load_index_double",
    )?;
    let index_offset_absolute = context.builder().build_int_add(
        index_double,
        context.field_const(
            crate::eravm::HEAP_AUX_OFFSET_CONSTRUCTOR_RETURN_DATA
                + (2 * era_compiler_common::BYTE_LENGTH_FIELD) as u64,
        ),
        "index_offset_absolute",
    )?;
    let index_offset_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::HeapAuxiliary,
        context.field_type(),
        index_offset_absolute,
        "immutable_index_pointer",
    )?;
    context.build_store(index_offset_pointer, index)?;

    let value_offset_absolute = context.builder().build_int_add(
        index_offset_absolute,
        context.field_const(era_compiler_common::BYTE_LENGTH_FIELD as u64),
        "value_offset_absolute",
    )?;
    let value_offset_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::HeapAuxiliary,
        context.field_type(),
        value_offset_absolute,
        "immutable_value_pointer",
    )?;
    context.build_store(value_offset_pointer, value)?;

    Ok(())
}
