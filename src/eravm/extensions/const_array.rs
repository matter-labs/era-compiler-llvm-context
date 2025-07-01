//!
//! Translates the const array instructions of the EraVM Yul extension.
//!

use inkwell::types::BasicType;
use inkwell::values::BasicValue;

use crate::context::IContext;
use crate::eravm::context::address_space::AddressSpace;
use crate::eravm::context::Context;

///
/// Declares a constant array in the code section.
///
pub fn declare<'ctx>(
    context: &mut Context<'ctx>,
    index: u8,
    size: u16,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    context
        .yul_mut()
        .expect("Always exists")
        .const_array_declare(index, size)?;

    Ok(context.field_const(1).as_basic_value_enum())
}

///
/// Sets a value in a constant array in the code section.
///
pub fn set<'ctx>(
    context: &mut Context<'ctx>,
    index: u8,
    offset: u16,
    value: num::BigUint,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    context
        .yul_mut()
        .expect("Always exists")
        .const_array_set(index, offset, value)?;

    Ok(context.field_const(1).as_basic_value_enum())
}

///
/// Finalizes a constant array in the code section, by extracting it from
/// the temporary compile-time storage, and initializing it in LLVM IR.
///
pub fn finalize<'ctx>(
    context: &mut Context<'ctx>,
    index: u8,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    let const_array = context
        .yul_mut()
        .expect("Always exists")
        .const_array_take(index)?;
    let array_type = context.field_type().array_type(const_array.len() as u32);
    let array_value = context.field_type().const_array(
        const_array
            .into_iter()
            .map(|value| context.field_const_str_dec(value.to_string().as_str()))
            .collect::<Vec<inkwell::values::IntValue<'ctx>>>()
            .as_slice(),
    );

    context.set_global(
        format!(
            "{}{:03}",
            crate::eravm::r#const::GLOBAL_CONST_ARRAY_PREFIX,
            index
        )
        .as_str(),
        array_type,
        AddressSpace::Code,
        array_value,
    )?;

    Ok(context.field_const(1).as_basic_value_enum())
}

///
/// Gets a value from a constant array in the code section.
///
pub fn get<'ctx>(
    context: &mut Context<'ctx>,
    index: u8,
    offset: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<inkwell::values::BasicValueEnum<'ctx>> {
    let identifier = format!(
        "{}{:03}",
        crate::eravm::r#const::GLOBAL_CONST_ARRAY_PREFIX,
        index
    );
    let global = context.get_global(identifier.as_str())?;
    let pointer = global.into();

    let pointer = context.build_gep(
        pointer,
        &[context.field_const(0), offset],
        context.field_type().as_basic_type_enum(),
        format!("{identifier}_pointer").as_str(),
    )?;
    let value = context.build_load(pointer, format!("{identifier}_value").as_str())?;

    Ok(value)
}
