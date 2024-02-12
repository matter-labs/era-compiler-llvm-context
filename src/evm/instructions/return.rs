//!
//! Translates the transaction return operations.
//!

use inkwell::values::BasicValue;

use crate::context::pointer::Pointer;
use crate::context::IContext;
use crate::evm::context::address_space::AddressSpace;
use crate::evm::context::Context;
use crate::evm::Dependency;

///
/// Translates the `return` instruction.
///
pub fn r#return<'ctx, D>(
    context: &mut Context<'ctx, D>,
    offset: inkwell::values::IntValue<'ctx>,
    length: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()>
where
    D: Dependency + Clone,
{
    let offset_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        offset,
        "revert_offset_pointer",
    );

    context.build_call(
        context.intrinsics().r#return,
        &[
            offset_pointer.as_basic_value_enum(),
            length.as_basic_value_enum(),
        ],
        "return",
    );
    context.build_unreachable();
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
    D: Dependency + Clone,
{
    let offset_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        offset,
        "revert_offset_pointer",
    );

    context.build_call(
        context.intrinsics().revert,
        &[
            offset_pointer.as_basic_value_enum(),
            length.as_basic_value_enum(),
        ],
        "revert",
    );
    context.build_unreachable();
    Ok(())
}

///
/// Translates the `stop` instruction.
///
pub fn stop<D>(context: &mut Context<D>) -> anyhow::Result<()>
where
    D: Dependency + Clone,
{
    context.build_call(context.intrinsics().stop, &[], "stop");
    context.build_unreachable();
    Ok(())
}

///
/// Translates the `invalid` instruction.
///
pub fn invalid<D>(context: &mut Context<D>) -> anyhow::Result<()>
where
    D: Dependency + Clone,
{
    context.build_call(context.intrinsics().invalid, &[], "invalid");
    context.build_unreachable();
    Ok(())
}
