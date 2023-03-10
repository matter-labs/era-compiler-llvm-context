//!
//! Translates the transaction return operations.
//!

use crate::context::address_space::AddressSpace;
use crate::context::code_type::CodeType;
use crate::context::pointer::Pointer;
use crate::context::Context;
use crate::Dependency;

///
/// Translates the `return` instruction.
///
/// Unlike in EVM, zkSync constructors return the array of contract immutables.
///
pub fn r#return<'ctx, D>(
    context: &mut Context<'ctx, D>,
    offset: inkwell::values::IntValue<'ctx>,
    length: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()>
where
    D: Dependency,
{
    match context.code_type() {
        None => {
            anyhow::bail!("Return is not available if the contract part is undefined");
        }
        Some(CodeType::Deploy) => {
            let immutables_offset_pointer = Pointer::new_with_offset(
                context,
                AddressSpace::HeapAuxiliary,
                context.field_type(),
                context.field_const(crate::HEAP_AUX_OFFSET_CONSTRUCTOR_RETURN_DATA),
                "immutables_offset_pointer",
            );
            context.build_store(
                immutables_offset_pointer,
                context.field_const(compiler_common::BYTE_LENGTH_FIELD as u64),
            );

            let immutables_number_pointer = Pointer::new_with_offset(
                context,
                AddressSpace::HeapAuxiliary,
                context.field_type(),
                context.field_const(
                    crate::HEAP_AUX_OFFSET_CONSTRUCTOR_RETURN_DATA
                        + (compiler_common::BYTE_LENGTH_FIELD as u64),
                ),
                "immutables_number_pointer",
            );
            let immutable_values_size = context.immutables_size()?;
            context.build_store(
                immutables_number_pointer,
                context.field_const(
                    (immutable_values_size / compiler_common::BYTE_LENGTH_FIELD) as u64,
                ),
            );
            let immutables_size = context.builder().build_int_mul(
                context.field_const(immutable_values_size as u64),
                context.field_const(2),
                "immutables_size",
            );
            let return_data_length = context.builder().build_int_add(
                immutables_size,
                context.field_const((compiler_common::BYTE_LENGTH_FIELD * 2) as u64),
                "return_data_length",
            );

            context.build_exit(
                context.intrinsics().r#return,
                context.field_const(crate::HEAP_AUX_OFFSET_CONSTRUCTOR_RETURN_DATA),
                return_data_length,
            );
        }
        Some(CodeType::Runtime) => {
            context.build_exit(context.intrinsics().r#return, offset, length);
        }
    }

    Ok(())
}

///
/// Translates the `revert` instruction.
///
pub fn revert<'ctx, D>(
    context: &mut Context<'ctx, D>,
    offset: inkwell::values::IntValue<'ctx>,
    length: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()>
where
    D: Dependency,
{
    context.build_exit(context.intrinsics().revert, offset, length);
    Ok(())
}

///
/// Translates the `stop` instruction.
///
/// Is the same as `return(0, 0)`.
///
pub fn stop<D>(context: &mut Context<D>) -> anyhow::Result<()>
where
    D: Dependency,
{
    r#return(context, context.field_const(0), context.field_const(0))
}

///
/// Translates the `invalid` instruction.
///
/// Is the same as `revert(0, 0)`.
///
pub fn invalid<D>(context: &mut Context<D>) -> anyhow::Result<()>
where
    D: Dependency,
{
    revert(context, context.field_const(0), context.field_const(0))
}
