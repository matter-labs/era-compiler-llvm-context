//!
//! The `keccak256` function.
//!

use inkwell::types::BasicType;
use inkwell::values::BasicValue;

use crate::context::address_space::AddressSpace;
use crate::context::function::Function;
use crate::context::pointer::Pointer;
use crate::context::Context;
use crate::Dependency;
use crate::WriteLLVM;

///
/// The `keccak256` function.
///
/// This instruction is implemented as a call to a system contract.
///
#[derive(Debug, Default)]
pub struct Keccak256 {}

impl Keccak256 {
    /// The default function name.
    pub const FUNCTION_NAME: &str = "__keccak256";

    /// The input offset argument index.
    pub const ARGUMENT_INDEX_INPUT_OFFSET: usize = 0;

    /// The input length argument index.
    pub const ARGUMENT_INDEX_INPUT_LENGTH: usize = 1;
}

impl<D> WriteLLVM<D> for Keccak256
where
    D: Dependency,
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

        Ok(())
    }

    fn into_llvm(self, context: &mut Context<D>) -> anyhow::Result<()> {
        context.set_current_function(Self::FUNCTION_NAME)?;

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

        let success_block = context.append_basic_block("keccak256_success_block");
        let failure_block = context.append_basic_block("keccak256_failure_block");

        context.set_basic_block(context.current_function().borrow().entry_block());
        let abi_data = crate::utils::abi_data(
            context,
            input_offset,
            input_length,
            None,
            AddressSpace::Heap,
            true,
        )?;
        let address = context.field_const(zkevm_opcode_defs::ADDRESS_KECCAK256.into());

        let result = context
            .build_call(
                context.llvm_runtime().static_call,
                crate::utils::external_call_arguments(
                    context,
                    abi_data.as_basic_value_enum(),
                    address,
                    vec![],
                    None,
                )
                .as_slice(),
                "keccak256_call_external",
            )
            .expect("Always returns a value");
        let result_abi_data_pointer = context
            .builder()
            .build_extract_value(
                result.into_struct_value(),
                0,
                "keccak256_call_external_result_abi_data_pointer",
            )
            .expect("Always valid");
        let result_abi_data_casted = Pointer::new(
            context.field_type(),
            AddressSpace::Generic,
            result_abi_data_pointer.into_pointer_value(),
        );

        let result_status_code_boolean = context
            .builder()
            .build_extract_value(
                result.into_struct_value(),
                1,
                "keccak256_external_result_status_code_boolean",
            )
            .expect("Always exists");
        let result_pointer = context.build_alloca(context.field_type(), "keccak256_result_pointer");
        context.build_store(result_pointer, context.field_const(0));
        context.build_conditional_branch(
            result_status_code_boolean.into_int_value(),
            success_block,
            failure_block,
        );

        context.set_basic_block(success_block);
        let child_data = context.build_load(result_abi_data_casted, "keccak256_child_data");
        context.build_store(result_pointer, child_data);
        context.build_unconditional_branch(context.current_function().borrow().return_block());

        context.set_basic_block(failure_block);
        if context
            .functions
            .contains_key(Function::ZKSYNC_NEAR_CALL_ABI_EXCEPTION_HANDLER)
        {
            crate::utils::throw(context)?;
        } else {
            crate::evm::r#return::revert(context, context.field_const(0), context.field_const(0))?;
        }

        context.set_basic_block(context.current_function().borrow().return_block());
        let result = context.build_load(result_pointer, "keccak256_result");
        context.build_return(Some(&result));

        Ok(())
    }
}
