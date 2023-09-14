//!
//! The `system_request` function.
//!

use inkwell::types::BasicType;
use inkwell::values::BasicValue;

use crate::eravm::context::address_space::AddressSpace;
use crate::eravm::context::function::Function;
use crate::eravm::context::pointer::Pointer;
use crate::eravm::context::Context;
use crate::eravm::Dependency;
use crate::eravm::WriteLLVM;

///
/// The `system_request` function.
///
#[derive(Debug, Default)]
pub struct SystemRequest {}

impl SystemRequest {
    /// The default function name.
    pub const FUNCTION_NAME: &str = "__system_request";

    /// The address argument index.
    pub const ARGUMENT_INDEX_ADDRESS: usize = 0;

    /// The input offset argument index.
    pub const ARGUMENT_INDEX_INPUT_OFFSET: usize = 1;

    /// The input length argument index.
    pub const ARGUMENT_INDEX_INPUT_LENGTH: usize = 2;
}

impl<D> WriteLLVM<D> for SystemRequest
where
    D: Dependency + Clone,
{
    fn declare(&mut self, context: &mut Context<D>) -> anyhow::Result<()> {
        let function_type = context.function_type(
            vec![
                context.field_type().as_basic_type_enum(),
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

        Ok(())
    }

    fn into_llvm(self, context: &mut Context<D>) -> anyhow::Result<()> {
        context.set_current_function(Self::FUNCTION_NAME)?;

        let address = context
            .current_function()
            .borrow()
            .get_nth_param(Self::ARGUMENT_INDEX_ADDRESS)
            .into_int_value();
        let input_offset = context
            .current_function()
            .borrow()
            .get_nth_param(Self::ARGUMENT_INDEX_INPUT_OFFSET)
            .into_int_value();
        let input_length = context
            .current_function()
            .borrow()
            .get_nth_param(Self::ARGUMENT_INDEX_INPUT_LENGTH)
            .into_int_value();

        let call_error_block = context.append_basic_block("system_request_error_block");

        context.set_basic_block(context.current_function().borrow().entry_block());
        let abi_data = crate::eravm::utils::abi_data(
            context,
            input_offset,
            input_length,
            None,
            AddressSpace::HeapAuxiliary,
            true,
        )?;
        let result = context
            .build_call(
                context.llvm_runtime().static_call,
                crate::eravm::utils::external_call_arguments(
                    context,
                    abi_data.as_basic_value_enum(),
                    address,
                    vec![],
                    None,
                )
                .as_slice(),
                "system_request",
            )
            .expect("Always returns a value");

        let result_abi_data_pointer = context
            .builder
            .build_extract_value(
                result.into_struct_value(),
                0,
                "system_request_result_abi_data",
            )
            .expect("Always exists");
        let result_abi_data_casted = Pointer::new(
            context.field_type(),
            AddressSpace::Generic,
            result_abi_data_pointer.into_pointer_value(),
        );

        let result_status_code_boolean = context
            .builder
            .build_extract_value(
                result.into_struct_value(),
                1,
                "system_request_result_status_code_boolean",
            )
            .expect("Always exists");
        let return_pointer =
            context.build_alloca(context.field_type(), "system_request_return_pointer");
        context.build_store(return_pointer, context.field_const(0));
        context.build_conditional_branch(
            result_status_code_boolean.into_int_value(),
            context.current_function().borrow().return_block(),
            call_error_block,
        );

        context.set_basic_block(call_error_block);
        if context
            .functions
            .contains_key(Function::ZKSYNC_NEAR_CALL_ABI_EXCEPTION_HANDLER)
        {
            crate::eravm::utils::throw(context)?;
        } else {
            crate::eravm::evm::r#return::revert(
                context,
                context.field_const(0),
                context.field_const(0),
            )?;
        }

        context.set_basic_block(context.current_function().borrow().return_block());
        let child_data_value =
            context.build_load(result_abi_data_casted, "system_request_child_address");
        context.build_return(Some(&child_data_value));

        Ok(())
    }
}
