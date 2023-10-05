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

    /// The signature argument index.
    pub const ARGUMENT_INDEX_SIGNATURE: usize = 1;

    /// The calldata size argument index.
    pub const ARGUMENT_INDEX_CALLDATA_SIZE: usize = 2;

    /// The calldata pointer index.
    pub const ARGUMENT_INDEX_CALLDATA_POINTER: usize = 3;
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
                context
                    .field_type()
                    .ptr_type(AddressSpace::Stack.into())
                    .as_basic_type_enum(),
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
        let signature = context
            .current_function()
            .borrow()
            .get_nth_param(Self::ARGUMENT_INDEX_SIGNATURE)
            .into_int_value();
        let calldata_size = context
            .current_function()
            .borrow()
            .get_nth_param(Self::ARGUMENT_INDEX_CALLDATA_SIZE)
            .into_int_value();
        let calldata_pointer = context
            .current_function()
            .borrow()
            .get_nth_param(Self::ARGUMENT_INDEX_CALLDATA_POINTER)
            .into_pointer_value();

        let calldata_loop_condition_block =
            context.append_basic_block("system_request_calldata_loop_condition_block");
        let calldata_loop_body_block =
            context.append_basic_block("system_request_calldata_loop_body_block");
        let calldata_loop_increment_block =
            context.append_basic_block("system_request_calldata_loop_increment_block");
        let calldata_loop_join_block =
            context.append_basic_block("system_request_calldata_loop_join_block");
        let call_error_block = context.append_basic_block("system_request_error_block");

        context.set_basic_block(context.current_function().borrow().entry_block());
        let signature_pointer = Pointer::new_with_offset(
            context,
            AddressSpace::HeapAuxiliary,
            context.field_type(),
            context.field_const(crate::eravm::HEAP_AUX_OFFSET_EXTERNAL_CALL),
            "system_request_signature_pointer",
        );
        context.build_store(signature_pointer, signature);

        let calldata_index_pointer = context.build_alloca(
            context.field_type(),
            "system_request_calldata_index_pointer",
        );
        context.build_store(
            calldata_index_pointer,
            context.field_const(compiler_common::BYTE_LENGTH_X32 as u64),
        );
        let stack_index_pointer =
            context.build_alloca(context.field_type(), "system_request_stack_index_pointer");
        context.build_store(stack_index_pointer, context.field_const(0));
        context.build_unconditional_branch(calldata_loop_condition_block);

        context.set_basic_block(calldata_loop_condition_block);
        let calldata_index_value = context.build_load(
            calldata_index_pointer,
            "system_request_calldata_index_value",
        );
        let calldata_condition = context.builder().build_int_compare(
            inkwell::IntPredicate::ULT,
            calldata_index_value.into_int_value(),
            calldata_size,
            "for_condition_compared",
        );
        context.build_conditional_branch(
            calldata_condition,
            calldata_loop_body_block,
            calldata_loop_join_block,
        );

        context.set_basic_block(calldata_loop_body_block);
        let stack_index_value =
            context.build_load(stack_index_pointer, "system_request_stack_index_value");
        let stack_pointer = Pointer::new_stack_field(context, calldata_pointer);
        let stack_pointer_with_offset = context.build_gep(
            stack_pointer,
            &[stack_index_value.into_int_value()],
            context.field_type(),
            "system_request_stack_pointer_with_offset",
        );
        let stack_value =
            context.build_load(stack_pointer_with_offset, "system_request_stack_value");
        let calldata_index_value = context.build_load(
            calldata_index_pointer,
            "system_request_calldata_index_value",
        );
        let calldata_pointer = Pointer::new_with_offset(
            context,
            AddressSpace::HeapAuxiliary,
            context.field_type(),
            calldata_index_value.into_int_value(),
            "system_request_calldata_pointer",
        );
        context.build_store(calldata_pointer, stack_value);
        context.build_unconditional_branch(calldata_loop_increment_block);

        context.set_basic_block(calldata_loop_increment_block);
        let calldata_index_value = context.build_load(
            calldata_index_pointer,
            "system_request_calldata_index_value",
        );
        let calldata_index_value_incremented = context.builder().build_int_add(
            calldata_index_value.into_int_value(),
            context.field_const(compiler_common::BYTE_LENGTH_FIELD as u64),
            "system_request_calldata_index_value_incremented",
        );
        context.build_store(calldata_index_pointer, calldata_index_value_incremented);

        let stack_index_value =
            context.build_load(stack_index_pointer, "system_request_stack_index_value");
        let stack_index_value_incremented = context.builder().build_int_add(
            stack_index_value.into_int_value(),
            context.field_const(1),
            "system_request_stack_index_value_incremented",
        );
        context.build_store(stack_index_pointer, stack_index_value_incremented);
        context.build_unconditional_branch(calldata_loop_condition_block);

        context.set_basic_block(calldata_loop_join_block);
        let abi_data = crate::eravm::utils::abi_data(
            context,
            context.field_const(crate::eravm::HEAP_AUX_OFFSET_EXTERNAL_CALL),
            calldata_size,
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
            crate::eravm::utils::throw(context);
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
