//!
//! The `exponent` function.
//!

use inkwell::types::BasicType;

use crate::eravm::context::function::Function;
use crate::eravm::context::Context;
use crate::eravm::Dependency;
use crate::eravm::WriteLLVM;

///
/// The `exponent` function.
///
#[derive(Debug, Default)]
pub struct Exponent {}

impl Exponent {
    /// The default function name.
    pub const FUNCTION_NAME: &str = "__exponent";

    /// The value argument index.
    pub const ARGUMENT_INDEX_VALUE: usize = 0;

    /// The exponent argument index.
    pub const ARGUMENT_INDEX_EXPONENT: usize = 1;
}

impl<D> WriteLLVM<D> for Exponent
where
    D: Dependency + Clone,
{
    fn declare(&mut self, context: &mut Context<D>) -> anyhow::Result<()> {
        let function_type = context.function_type(
            vec![
                context.field_type().as_basic_type_enum(),
                context.field_type().as_basic_type_enum(),
            ],
            1,
            false,
        );
        let function = context.add_function(
            Self::FUNCTION_NAME,
            function_type,
            1,
            Some(inkwell::module::Linkage::Private),
        )?;
        Function::set_frontend_runtime_attributes(
            context.llvm,
            function.borrow().declaration(),
            &context.optimizer,
        );
        Function::set_llvm_runtime_attributes(context.llvm, function.borrow().declaration());

        Ok(())
    }

    fn into_llvm(self, context: &mut Context<D>) -> anyhow::Result<()> {
        context.set_current_function(Self::FUNCTION_NAME)?;

        let value = context
            .current_function()
            .borrow()
            .get_nth_param(Self::ARGUMENT_INDEX_VALUE)
            .into_int_value();
        let exponent = context
            .current_function()
            .borrow()
            .get_nth_param(Self::ARGUMENT_INDEX_EXPONENT)
            .into_int_value();

        let condition_block = context.append_basic_block("exponent_loop_condition");
        let body_block = context.append_basic_block("exponent_loop_body");
        let multiplying_block = context.append_basic_block("exponent_loop_multiplying");
        let iterating_block = context.append_basic_block("exponent_loop_iterating");

        context.set_basic_block(context.current_function().borrow().entry_block());
        let factor_pointer = context.build_alloca(context.field_type(), "exponent_factor");
        context.build_store(factor_pointer, value);
        let power_pointer =
            context.build_alloca(context.field_type(), "exponent_loop_power_pointer");
        context.build_store(power_pointer, exponent);
        let result_pointer = context.build_alloca(context.field_type(), "exponent_result");
        context.build_store(result_pointer, context.field_const(1));
        context.build_unconditional_branch(condition_block);

        context.set_basic_block(condition_block);
        let power_value = context
            .build_load(power_pointer, "exponent_loop_power_value_condition")
            .into_int_value();
        let condition = context.builder().build_int_compare(
            inkwell::IntPredicate::UGT,
            power_value,
            context.field_const(0),
            "exponent_loop_is_power_value_non_zero",
        );
        context.build_conditional_branch(
            condition,
            body_block,
            context.current_function().borrow().return_block(),
        );

        context.set_basic_block(iterating_block);
        let factor_value = context
            .build_load(factor_pointer, "exponent_loop_factor_value_to_square")
            .into_int_value();
        let factor_value_squared = context.builder().build_int_mul(
            factor_value,
            factor_value,
            "exponent_loop_factor_value_squared",
        );
        context.build_store(factor_pointer, factor_value_squared);
        let power_value = context
            .build_load(power_pointer, "exponent_loop_power_value_to_half")
            .into_int_value();
        let power_value_halved = context.builder().build_int_unsigned_div(
            power_value,
            context.field_const(2),
            "exponent_loop_power_value_halved",
        );
        context.build_store(power_pointer, power_value_halved);
        context.build_unconditional_branch(condition_block);

        context.set_basic_block(body_block);
        let power_value = context
            .build_load(power_pointer, "exponent_loop_power_value_body")
            .into_int_value();
        let power_lowest_bit = context.builder().build_int_truncate_or_bit_cast(
            power_value,
            context.bool_type(),
            "exponent_loop_power_body_lowest_bit",
        );
        context.build_conditional_branch(power_lowest_bit, multiplying_block, iterating_block);

        context.set_basic_block(multiplying_block);
        let intermediate = context
            .build_load(result_pointer, "exponent_loop_intermediate_result")
            .into_int_value();
        let factor_value = context
            .build_load(factor_pointer, "exponent_loop_intermediate_factor_value")
            .into_int_value();
        let result = context.builder().build_int_mul(
            intermediate,
            factor_value,
            "exponent_loop_intermediate_result_multiplied",
        );
        context.build_store(result_pointer, result);
        context.build_unconditional_branch(iterating_block);

        context.set_basic_block(context.current_function().borrow().return_block());
        let result = context.build_load(result_pointer, "exponent_result");
        context.build_return(Some(&result));

        Ok(())
    }
}
