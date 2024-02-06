//!
//! Translates the return data instructions.
//!

use crate::context::pointer::Pointer;
use crate::context::IContext;
use crate::evm::context::address_space::AddressSpace;
use crate::evm::context::Context;
use crate::evm::Dependency;

///
/// Translates the return data size.
///
pub fn size<'ctx, D>(
    context: &mut Context<'ctx, D>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>>
where
    D: Dependency + Clone,
{
    Ok(context
        .build_call(context.intrinsics().returndatasize, &[], "returndatasize")
        .expect("Always exists"))
}

///
/// Translates the return data copy.
///
pub fn copy<'ctx, D>(
    context: &mut Context<'ctx, D>,
    destination_offset: inkwell::values::IntValue<'ctx>,
    source_offset: inkwell::values::IntValue<'ctx>,
    size: inkwell::values::IntValue<'ctx>,
) where
    D: Dependency + Clone,
{
    let destination = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        destination_offset,
        "returndatacopy_destination_pointer",
    );

    let source = Pointer::new_with_offset(
        context,
        AddressSpace::ReturnData,
        context.byte_type(),
        source_offset,
        "returndatacopy_source_pointer",
    );

    context.build_memcpy(
        context.intrinsics().memory_copy_from_return_data,
        destination,
        source,
        size,
        "returndatacopy_memcpy",
    );
}
