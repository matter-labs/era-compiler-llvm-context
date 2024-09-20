//!
//! The `deployer_call` function.
//!

use inkwell::types::BasicType;
use inkwell::values::BasicValue;

use crate::context::pointer::Pointer;
use crate::context::IContext;
use crate::eravm::context::address_space::AddressSpace;
use crate::eravm::context::function::Function;
use crate::eravm::context::Context;
use crate::eravm::Dependency;
use crate::eravm::WriteLLVM;

///
/// The `deployer_call` function.
///
/// Calls the deployer system contract, which returns the newly deployed contract address or 0.
///
/// The address is returned in the first 32-byte word of the return data. If it is 0, the 0 is
/// returned. If the entire call has failed, there is also a 0 returned.
///
#[derive(Debug)]
pub struct DeployerCall {
    /// The address space where the calldata is allocated.
    /// Solidity uses the ordinary heap. Vyper uses the auxiliary heap.
    address_space: AddressSpace,
}

impl DeployerCall {
    /// The value argument index.
    pub const ARGUMENT_INDEX_VALUE: usize = 0;

    /// The input offset argument index.
    pub const ARGUMENT_INDEX_INPUT_OFFSET: usize = 1;

    /// The input length argument index.
    pub const ARGUMENT_INDEX_INPUT_LENGTH: usize = 2;

    /// The signature hash argument index.
    pub const ARGUMENT_INDEX_SIGNATURE_HASH: usize = 3;

    /// The salt argument index.
    pub const ARGUMENT_INDEX_SALT: usize = 4;

    /// The heap function name.
    const FUNCTION_NAME: &'static str = "__deployer_call";

    ///
    /// A shortcut constructor.
    ///
    pub fn new(address_space: AddressSpace) -> Self {
        Self { address_space }
    }

    ///
    /// Returns the function name based on its address space.
    ///
    pub fn name(address_space: AddressSpace) -> String {
        match address_space {
            AddressSpace::HeapAuxiliary => format!("{}_aux", Self::FUNCTION_NAME),
            AddressSpace::Heap => Self::FUNCTION_NAME.to_owned(),
            _ => panic!("DeployerCall can only use Heap or HeapAuxiliary address spaces"),
        }
    }
}

impl<D> WriteLLVM<D> for DeployerCall
where
    D: Dependency,
{
    fn declare(&mut self, context: &mut Context<D>) -> anyhow::Result<()> {
        let function_type = context.function_type(
            vec![
                context.field_type().as_basic_type_enum(),
                context.field_type().as_basic_type_enum(),
                context.field_type().as_basic_type_enum(),
                context.field_type().as_basic_type_enum(),
                context.field_type().as_basic_type_enum(),
            ],
            1,
            false,
        );
        let function = context.add_function(
            Self::name(self.address_space).as_str(),
            function_type,
            1,
            Some(inkwell::module::Linkage::Private),
        )?;
        Function::set_frontend_runtime_attributes(
            context.llvm,
            function.borrow().declaration().value,
            &context.optimizer,
        );

        Ok(())
    }

    fn into_llvm(self, context: &mut Context<D>) -> anyhow::Result<()> {
        context.set_current_function(Self::name(self.address_space).as_str())?;

        let value = context
            .current_function()
            .borrow()
            .get_nth_param(Self::ARGUMENT_INDEX_VALUE)
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
        let signature_hash = context
            .current_function()
            .borrow()
            .get_nth_param(Self::ARGUMENT_INDEX_SIGNATURE_HASH)
            .into_int_value();
        let salt = context
            .current_function()
            .borrow()
            .get_nth_param(Self::ARGUMENT_INDEX_SALT)
            .into_int_value();

        let error_block = context.append_basic_block("deployer_call_error_block");
        let success_block = context.append_basic_block("deployer_call_success_block");
        let value_zero_block = context.append_basic_block("deployer_call_value_zero_block");
        let value_non_zero_block = context.append_basic_block("deployer_call_value_non_zero_block");
        let value_join_block = context.append_basic_block("deployer_call_value_join_block");

        context.set_basic_block(context.current_function().borrow().entry_block());
        let abi_data = crate::eravm::utils::abi_data(
            context,
            input_offset,
            input_length,
            None,
            self.address_space,
            true,
        )?;

        let signature_pointer = Pointer::new_with_offset(
            context,
            self.address_space,
            context.field_type(),
            input_offset,
            "deployer_call_signature_pointer",
        )?;
        context.build_store(signature_pointer, signature_hash)?;

        let salt_offset = context.builder().build_int_add(
            input_offset,
            context.field_const(era_compiler_common::BYTE_LENGTH_X32 as u64),
            "deployer_call_salt_offset",
        )?;
        let salt_pointer = Pointer::new_with_offset(
            context,
            self.address_space,
            context.field_type(),
            salt_offset,
            "deployer_call_salt_pointer",
        )?;
        context.build_store(salt_pointer, salt)?;

        let arguments_offset_offset = context.builder().build_int_add(
            salt_offset,
            context.field_const((era_compiler_common::BYTE_LENGTH_FIELD * 2) as u64),
            "deployer_call_arguments_offset_offset",
        )?;
        let arguments_offset_pointer = Pointer::new_with_offset(
            context,
            self.address_space,
            context.field_type(),
            arguments_offset_offset,
            "deployer_call_arguments_offset_pointer",
        )?;
        context.build_store(
            arguments_offset_pointer,
            context.field_const(
                (crate::eravm::DEPLOYER_CALL_HEADER_SIZE
                    - (era_compiler_common::BYTE_LENGTH_X32
                        + era_compiler_common::BYTE_LENGTH_FIELD)) as u64,
            ),
        )?;

        let arguments_length_offset = context.builder().build_int_add(
            arguments_offset_offset,
            context.field_const(era_compiler_common::BYTE_LENGTH_FIELD as u64),
            "deployer_call_arguments_length_offset",
        )?;
        let arguments_length_pointer = Pointer::new_with_offset(
            context,
            self.address_space,
            context.field_type(),
            arguments_length_offset,
            "deployer_call_arguments_length_pointer",
        )?;
        let arguments_length_value = context.builder().build_int_sub(
            input_length,
            context.field_const(crate::eravm::DEPLOYER_CALL_HEADER_SIZE as u64),
            "deployer_call_arguments_length",
        )?;
        context.build_store(arguments_length_pointer, arguments_length_value)?;

        let result_pointer =
            context.build_alloca(context.field_type(), "deployer_call_result_pointer")?;
        context.build_store(result_pointer, context.field_const(0))?;
        let deployer_call_result_type = context.structure_type(&[
            context
                .ptr_type(AddressSpace::Generic.into())
                .as_basic_type_enum(),
            context.bool_type().as_basic_type_enum(),
        ]);
        let deployer_call_result_pointer =
            context.build_alloca(deployer_call_result_type, "deployer_call_result_pointer")?;
        context.build_store(
            deployer_call_result_pointer,
            deployer_call_result_type.const_zero(),
        )?;
        let is_value_zero = context.builder().build_int_compare(
            inkwell::IntPredicate::EQ,
            value,
            context.field_const(0),
            "deployer_call_is_value_zero",
        )?;
        context.build_conditional_branch(is_value_zero, value_zero_block, value_non_zero_block)?;

        context.set_basic_block(value_zero_block);
        let deployer_call_result = context
            .build_call(
                context.llvm_runtime().far_call,
                crate::eravm::utils::external_call_arguments(
                    context,
                    abi_data,
                    context.field_const(zkevm_opcode_defs::ADDRESS_CONTRACT_DEPLOYER.into()),
                    vec![],
                    None,
                )
                .as_slice(),
                "deployer_call_ordinary",
            )?
            .expect("Always returns a value");
        context.build_store(deployer_call_result_pointer, deployer_call_result)?;
        context.build_unconditional_branch(value_join_block)?;

        context.set_basic_block(value_non_zero_block);
        let deployer_call_result = context
            .build_call(
                context.llvm_runtime().far_call,
                crate::eravm::utils::external_call_arguments(
                    context,
                    abi_data.as_basic_value_enum(),
                    context.field_const(zkevm_opcode_defs::ADDRESS_MSG_VALUE.into()),
                    vec![
                        value,
                        context.field_const(zkevm_opcode_defs::ADDRESS_CONTRACT_DEPLOYER.into()),
                        context.field_const(u64::from(crate::eravm::r#const::SYSTEM_CALL_BIT)),
                    ],
                    None,
                )
                .as_slice(),
                "deployer_call_system",
            )?
            .expect("Always returns a value");
        context.build_store(deployer_call_result_pointer, deployer_call_result)?;
        context.build_unconditional_branch(value_join_block)?;

        context.set_basic_block(value_join_block);
        let result_abi_data_pointer = context.build_gep(
            deployer_call_result_pointer,
            &[
                context.field_const(0),
                context
                    .integer_type(era_compiler_common::BIT_LENGTH_X32)
                    .const_zero(),
            ],
            context
                .ptr_type(AddressSpace::Generic.into())
                .as_basic_type_enum(),
            "deployer_call_result_abi_data_pointer",
        )?;
        let result_abi_data =
            context.build_load(result_abi_data_pointer, "deployer_call_result_abi_data")?;

        let result_status_code_pointer = context.build_gep(
            deployer_call_result_pointer,
            &[
                context.field_const(0),
                context
                    .integer_type(era_compiler_common::BIT_LENGTH_X32)
                    .const_int(1, false),
            ],
            context.bool_type().as_basic_type_enum(),
            "contract_call_external_result_status_code_pointer",
        )?;
        let result_status_code_boolean = context
            .build_load(
                result_status_code_pointer,
                "contract_call_external_result_status_code_boolean",
            )?
            .into_int_value();

        context.build_conditional_branch(result_status_code_boolean, success_block, error_block)?;

        context.set_basic_block(success_block);
        let result_abi_data_pointer = Pointer::new(
            context.field_type(),
            AddressSpace::Generic,
            result_abi_data.into_pointer_value(),
        );
        let address_or_status_code = context.build_load(
            result_abi_data_pointer,
            "deployer_call_address_or_status_code",
        )?;
        context.build_store(result_pointer, address_or_status_code)?;
        context.build_unconditional_branch(context.current_function().borrow().return_block())?;

        context.set_basic_block(error_block);
        let result_abi_data_pointer = Pointer::new(
            context.byte_type(),
            AddressSpace::Generic,
            result_abi_data.into_pointer_value(),
        );
        context.write_abi_pointer(
            result_abi_data_pointer,
            crate::eravm::GLOBAL_RETURN_DATA_POINTER,
        )?;
        context.write_abi_data_size(
            result_abi_data_pointer,
            crate::eravm::GLOBAL_RETURN_DATA_SIZE,
        )?;
        context.build_unconditional_branch(context.current_function().borrow().return_block())?;

        context.set_basic_block(context.current_function().borrow().return_block());
        let result = context.build_load(result_pointer, "deployer_call_result")?;
        context.build_return(Some(&result))?;

        Ok(())
    }
}
