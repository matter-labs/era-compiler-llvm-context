//!
//! Translates the contract immutable operations.
//!

use crate::context::pointer::Pointer;
use crate::context::traits::solidity_data::ISolidityData;
use crate::context::IContext;
use crate::evm::context::address_space::AddressSpace;
use crate::evm::context::Context;

///
/// Translates the `loadimmutable` instruction.
///
pub fn load<'ctx>(
    context: &mut Context<'ctx>,
    id: &str,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    let id = context
        .llvm()
        .metadata_node(&[context.llvm().metadata_string(id).into()]);

    Ok(context
        .build_call_metadata(
            context.intrinsics().loadimmutable,
            &[id.into()],
            "load_immutable",
        )?
        .expect("Always exists"))
}

///
/// Translates the `setimmutable` instruction.
///
pub fn store<'ctx>(
    context: &mut Context<'ctx>,
    id: &str,
    base_offset: inkwell::values::IntValue<'ctx>,
    value: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()> {
    let offsets = match context.solidity().expect("Always exists").offsets(id) {
        Some(offsets) => offsets,
        None if id == crate::r#const::LIBRARY_DEPLOY_ADDRESS_TAG => return Ok(()),
        None => anyhow::bail!("Undefined immutable identifier: {id}"),
    };

    for offset in offsets.iter() {
        let immutable_offset = context.builder().build_int_add(
            base_offset,
            context.field_const(*offset),
            "setimmutable_offset",
        )?;
        let immutable_pointer = Pointer::new_with_offset(
            context,
            AddressSpace::Heap,
            context.byte_type(),
            immutable_offset,
            "setimmutable_pointer",
        )?;
        context.build_store(immutable_pointer, value)?;
    }

    Ok(())
}
