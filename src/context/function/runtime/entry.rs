//!
//! The entry function.
//!

use inkwell::types::BasicType;

use crate::context::address_space::AddressSpace;
use crate::context::function::runtime::Runtime;
use crate::context::pointer::Pointer;
use crate::context::Context;
use crate::Dependency;
use crate::WriteLLVM;

///
/// The entry function.
///
/// The function is a wrapper managing the runtime and deploy code calling logic.
///
/// Is a special runtime function that is only used by the front-end generated code.
///
#[derive(Debug, Default)]
pub struct Entry {}

impl Entry {
    /// The calldata ABI argument index.
    pub const ARGUMENT_INDEX_CALLDATA_ABI: usize = 0;

    /// The call flags argument index.
    pub const ARGUMENT_INDEX_CALL_FLAGS: usize = 1;

    /// The number of mandatory arguments.
    pub const MANDATORY_ARGUMENTS_COUNT: usize = 2;

    ///
    /// Initializes the global variables.
    ///
    /// The pointers are not initialized, because it's not possible to create a null pointer.
    ///
    pub fn initialize_globals<D>(context: &mut Context<D>) -> anyhow::Result<()>
    where
        D: Dependency,
    {
        context.set_global(
            crate::GLOBAL_CALLDATA_SIZE,
            context.field_type(),
            context.field_const(0),
        );
        context.set_global(
            crate::GLOBAL_RETURN_DATA_SIZE,
            context.field_type(),
            context.field_const(0),
        );

        context.set_global(
            crate::GLOBAL_CALL_FLAGS,
            context.field_type(),
            context.field_const(0),
        );

        let extra_abi_data_type = context.array_type(
            context.field_type().as_basic_type_enum(),
            crate::EXTRA_ABI_DATA_SIZE,
        );
        context.set_global(
            crate::GLOBAL_EXTRA_ABI_DATA,
            extra_abi_data_type,
            extra_abi_data_type.const_zero(),
        );

        Ok(())
    }
}

impl<D> WriteLLVM<D> for Entry
where
    D: Dependency,
{
    fn declare(&mut self, context: &mut Context<D>) -> anyhow::Result<()> {
        let mut entry_arguments =
            Vec::with_capacity(Self::MANDATORY_ARGUMENTS_COUNT + crate::EXTRA_ABI_DATA_SIZE);
        entry_arguments.push(
            context
                .byte_type()
                .ptr_type(AddressSpace::Generic.into())
                .as_basic_type_enum(),
        );
        entry_arguments.push(context.field_type().as_basic_type_enum());
        entry_arguments.extend(vec![
            context.field_type().as_basic_type_enum();
            crate::EXTRA_ABI_DATA_SIZE
        ]);
        let function_type = context.function_type(entry_arguments, 1, false);
        context.add_function(
            Runtime::FUNCTION_ENTRY,
            function_type,
            0,
            Some(inkwell::module::Linkage::External),
        )?;

        Ok(())
    }

    fn into_llvm(self, context: &mut Context<D>) -> anyhow::Result<()> {
        context.set_current_function(Runtime::FUNCTION_ENTRY)?;

        let deploy_code_call_block = context.append_basic_block("deploy_code_call_block");
        let runtime_code_call_block = context.append_basic_block("runtime_code_call_block");

        let deploy_code = context
            .functions
            .get(Runtime::FUNCTION_DEPLOY_CODE)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Contract deploy code not found"))?;
        let runtime_code = context
            .functions
            .get(Runtime::FUNCTION_RUNTIME_CODE)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Contract runtime code not found"))?;

        context.set_basic_block(context.current_function().borrow().entry_block());
        Self::initialize_globals(context)?;

        let calldata_abi = context
            .current_function()
            .borrow()
            .get_nth_param(Self::ARGUMENT_INDEX_CALLDATA_ABI);
        let calldata_abi_pointer = Pointer::new(
            context.byte_type(),
            AddressSpace::Generic,
            calldata_abi.into_pointer_value(),
        );
        context.write_abi_pointer(calldata_abi_pointer, crate::GLOBAL_CALLDATA_POINTER);
        context.write_abi_data_size(calldata_abi_pointer, crate::GLOBAL_CALLDATA_SIZE);
        let calldata_length = context.get_global(crate::GLOBAL_CALLDATA_SIZE)?;
        let calldata_end_pointer = context.build_gep(
            calldata_abi_pointer,
            &[calldata_length.into_int_value()],
            context
                .byte_type()
                .ptr_type(AddressSpace::Generic.into())
                .as_basic_type_enum(),
            "return_data_abi_initializer",
        );
        context.write_abi_pointer(calldata_end_pointer, crate::GLOBAL_RETURN_DATA_POINTER);
        context.write_abi_pointer(calldata_end_pointer, crate::GLOBAL_ACTIVE_POINTER);

        let call_flags = context
            .current_function()
            .borrow()
            .get_nth_param(Self::ARGUMENT_INDEX_CALL_FLAGS);
        context.set_global(
            crate::GLOBAL_CALL_FLAGS,
            call_flags.get_type(),
            call_flags.into_int_value(),
        );

        let extra_abi_data_pointer = context.get_global_ptr(crate::GLOBAL_EXTRA_ABI_DATA)?;
        for (array_index, argument_index) in (Self::MANDATORY_ARGUMENTS_COUNT
            ..Self::MANDATORY_ARGUMENTS_COUNT + crate::EXTRA_ABI_DATA_SIZE)
            .into_iter()
            .enumerate()
        {
            let array_element_pointer = context.build_gep(
                extra_abi_data_pointer,
                &[
                    context.field_const(0),
                    context
                        .integer_type(compiler_common::BIT_LENGTH_X32)
                        .const_int(array_index as u64, false),
                ],
                context.field_type().as_basic_type_enum(),
                "extra_abi_data_array_element_pointer",
            );
            let argument_value = context
                .current_function()
                .borrow()
                .get_nth_param(argument_index)
                .into_int_value();
            context.build_store(array_element_pointer, argument_value);
        }

        let is_deploy_call_flag_truncated = context.builder().build_and(
            call_flags.into_int_value(),
            context.field_const(1),
            "is_deploy_code_call_flag_truncated",
        );
        let is_deploy_code_call_flag = context.builder().build_int_compare(
            inkwell::IntPredicate::EQ,
            is_deploy_call_flag_truncated,
            context.field_const(1),
            "is_deploy_code_call_flag",
        );
        context.build_conditional_branch(
            is_deploy_code_call_flag,
            deploy_code_call_block,
            runtime_code_call_block,
        );

        context.set_basic_block(deploy_code_call_block);
        context.build_invoke(deploy_code.borrow().declaration, &[], "deploy_code_call");
        context.build_unconditional_branch(context.current_function().borrow().return_block());

        context.set_basic_block(runtime_code_call_block);
        context.build_invoke(runtime_code.borrow().declaration, &[], "runtime_code_call");
        context.build_unconditional_branch(context.current_function().borrow().return_block());

        context.set_basic_block(context.current_function().borrow().return_block());
        context.build_return(Some(&context.field_const(0)));

        Ok(())
    }
}
