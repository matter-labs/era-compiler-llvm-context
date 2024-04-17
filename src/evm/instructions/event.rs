//!
//! Translates a log or event call.
//!

use inkwell::values::BasicValue;

use crate::context::pointer::Pointer;
use crate::context::IContext;
use crate::evm::context::address_space::AddressSpace;
use crate::evm::context::Context;
use crate::evm::Dependency;

///
/// Translates an event log call.
///
pub fn log<'ctx, D>(
    context: &mut Context<'ctx, D>,
    topics: Vec<inkwell::values::IntValue<'ctx>>,
    input_offset: inkwell::values::IntValue<'ctx>,
    input_length: inkwell::values::IntValue<'ctx>,
) -> anyhow::Result<()>
where
    D: Dependency + Clone,
{
    let input_offset_pointer = Pointer::new_with_offset(
        context,
        AddressSpace::Heap,
        context.byte_type(),
        input_offset,
        format!("log{}_input_offset_pointer", topics.len()).as_str(),
    )?;

    match topics.len() {
        0 => context.build_call(
            context.intrinsics().log0,
            &[
                input_offset_pointer.as_basic_value_enum(),
                input_length.as_basic_value_enum(),
            ],
            "log0",
        ),
        1 => context.build_call(
            context.intrinsics().log1,
            &[
                topics[0].as_basic_value_enum(),
                input_offset_pointer.as_basic_value_enum(),
                input_length.as_basic_value_enum(),
            ],
            "log1",
        ),
        2 => context.build_call(
            context.intrinsics().log2,
            &[
                topics[0].as_basic_value_enum(),
                topics[1].as_basic_value_enum(),
                input_offset_pointer.as_basic_value_enum(),
                input_length.as_basic_value_enum(),
            ],
            "log2",
        ),
        3 => context.build_call(
            context.intrinsics().log3,
            &[
                topics[0].as_basic_value_enum(),
                topics[1].as_basic_value_enum(),
                topics[2].as_basic_value_enum(),
                input_offset_pointer.as_basic_value_enum(),
                input_length.as_basic_value_enum(),
            ],
            "log3",
        ),
        4 => context.build_call(
            context.intrinsics().log4,
            &[
                topics[0].as_basic_value_enum(),
                topics[1].as_basic_value_enum(),
                topics[2].as_basic_value_enum(),
                topics[3].as_basic_value_enum(),
                input_offset_pointer.as_basic_value_enum(),
                input_length.as_basic_value_enum(),
            ],
            "log4",
        ),
        length => panic!("The number of topics must be from 0 to 4, found {}", length),
    }?;
    Ok(())
}
