//!
//! Translates the transaction return operations.
//!

use crate::context::pointer::Pointer;
use crate::context::IContext;
use crate::eravm::context::address_space::AddressSpace;
use crate::eravm::context::Context;

///
/// Translates the `return` instruction.
///
/// Unlike in EVM, ZKsync constructors return the array of contract immutables.
///
pub fn r#return<'ctx>(
    context: &mut Context<'ctx>,
    offset: inkwell::values::IntValue<'ctx>,
    length: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()> {
    match context.code_segment() {
        None => {
            anyhow::bail!("Contract code segment is undefined");
        }
        Some(era_compiler_common::CodeSegment::Deploy) => {
            let immutables_offset_pointer = Pointer::new_with_offset(
                context,
                AddressSpace::HeapAuxiliary,
                context.field_type(),
                context.field_const(crate::eravm::HEAP_AUX_OFFSET_CONSTRUCTOR_RETURN_DATA),
                "immutables_offset_pointer",
            )?;
            context.build_store(
                immutables_offset_pointer,
                context.field_const(era_compiler_common::BYTE_LENGTH_FIELD as u64),
            )?;

            let immutables_number_pointer = Pointer::new_with_offset(
                context,
                AddressSpace::HeapAuxiliary,
                context.field_type(),
                context.field_const(
                    crate::eravm::HEAP_AUX_OFFSET_CONSTRUCTOR_RETURN_DATA
                        + (era_compiler_common::BYTE_LENGTH_FIELD as u64),
                ),
                "immutables_number_pointer",
            )?;
            let immutable_values_size = context.immutables_size();
            context.build_store(
                immutables_number_pointer,
                context.field_const(
                    (immutable_values_size / era_compiler_common::BYTE_LENGTH_FIELD) as u64,
                ),
            )?;
            let immutables_size = context.builder().build_int_mul(
                context.field_const(immutable_values_size as u64),
                context.field_const(2),
                "immutables_size",
            )?;
            let return_data_length = context.builder().build_int_add(
                immutables_size,
                context.field_const((era_compiler_common::BYTE_LENGTH_FIELD * 2) as u64),
                "return_data_length",
            )?;

            context.build_exit(
                context.llvm_runtime().r#return,
                context.field_const(crate::eravm::HEAP_AUX_OFFSET_CONSTRUCTOR_RETURN_DATA),
                return_data_length,
            )?;
        }
        Some(era_compiler_common::CodeSegment::Runtime) => {
            context.build_exit(context.llvm_runtime().r#return, offset, length)?;
        }
    }

    Ok(())
}

///
/// Translates the `revert` instruction.
///
pub fn revert<'ctx>(
    context: &mut Context<'ctx>,
    offset: inkwell::values::IntValue<'ctx>,
    length: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()> {
    context.build_exit(context.llvm_runtime().revert, offset, length)?;
    Ok(())
}

///
/// Translates the `stop` instruction.
///
/// Is the same as `return(0, 0)`.
///
pub fn stop(context: &mut Context) -> anyhow::Result<()> {
    r#return(context, context.field_const(0), context.field_const(0))
}

///
/// Translates the `invalid` instruction.
///
/// Burns all gas using an out-of-bounds memory store, causing a panic.
///
pub fn invalid(context: &mut Context) -> anyhow::Result<()> {
    crate::eravm::evm::memory::store(
        context,
        context.field_type().const_all_ones(),
        context.field_const(0),
    )?;
    context.build_call(context.intrinsics().trap, &[], "invalid_trap")?;
    context.build_unreachable()?;
    Ok(())
}
