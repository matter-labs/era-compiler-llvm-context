//!
//! Translates the cryptographic operations.
//!

use inkwell::values::BasicValue;

use crate::context::IContext;
use crate::eravm::context::address_space::AddressSpace;
use crate::eravm::context::function::Function as EraVMFunction;
use crate::eravm::context::Context;

///
/// Translates the `sha3` instruction.
///
pub fn sha3<'ctx>(
    context: &mut Context<'ctx>,
    offset: inkwell::values::IntValue<'ctx>,
    length: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    let offset_pointer = context.builder().build_int_to_ptr(
        offset,
        context.ptr_type(AddressSpace::Heap.into()),
        "sha3_offset_pointer",
    )?;

    Ok(context
        .build_invoke(
            context.llvm_runtime().sha3,
            &[
                offset_pointer.as_basic_value_enum(),
                length.as_basic_value_enum(),
                context
                    .bool_const(
                        context
                            .get_function(EraVMFunction::ZKSYNC_NEAR_CALL_ABI_EXCEPTION_HANDLER)
                            .is_some(),
                    )
                    .as_basic_value_enum(),
            ],
            "sha3_call",
        )?
        .expect("Always exists"))
}
