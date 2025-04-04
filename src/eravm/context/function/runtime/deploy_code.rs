//!
//! The deploy code function.
//!

use crate::context::pointer::Pointer;
use crate::context::IContext;
use crate::eravm::context::address_space::AddressSpace;
use crate::eravm::context::function::runtime::Runtime;
use crate::eravm::context::Context;
use crate::eravm::WriteLLVM;

///
/// The deploy code function.
///
/// Is a special function that is only used by the front-end generated code.
///
#[derive(Debug)]
pub struct DeployCode<B>
where
    B: WriteLLVM,
{
    /// The deploy code AST representation.
    inner: B,
}

impl<B> DeployCode<B>
where
    B: WriteLLVM,
{
    ///
    /// A shortcut constructor.
    ///
    pub fn new(inner: B) -> Self {
        Self { inner }
    }
}

impl<B> WriteLLVM for DeployCode<B>
where
    B: WriteLLVM,
{
    fn declare(&mut self, context: &mut Context) -> anyhow::Result<()> {
        let function_type =
            context.function_type::<inkwell::types::BasicTypeEnum>(vec![], 0, false);
        context.add_function(
            Runtime::FUNCTION_DEPLOY_CODE,
            function_type,
            0,
            Some(inkwell::module::Linkage::Private),
        )?;

        self.inner.declare(context)
    }

    fn into_llvm(self, context: &mut Context) -> anyhow::Result<()> {
        context.set_current_function(Runtime::FUNCTION_DEPLOY_CODE)?;

        context.set_basic_block(context.current_function().borrow().entry_block());
        context.set_code_segment(era_compiler_common::CodeSegment::Deploy);
        if let Some(vyper) = context.vyper_data.as_ref() {
            for index in 0..vyper.immutables_size() / era_compiler_common::BYTE_LENGTH_FIELD {
                let offset = (crate::eravm::r#const::HEAP_AUX_OFFSET_CONSTRUCTOR_RETURN_DATA
                    as usize)
                    + (1 + index) * 2 * era_compiler_common::BYTE_LENGTH_FIELD;
                let value = index * era_compiler_common::BYTE_LENGTH_FIELD;
                let pointer = Pointer::new_with_offset(
                    context,
                    AddressSpace::HeapAuxiliary,
                    context.field_type(),
                    context.field_const(offset as u64),
                    "immutable_index_initializer",
                )?;
                context.build_store(pointer, context.field_const(value as u64))?;
            }
        }

        self.inner.into_llvm(context)?;
        match context
            .basic_block()
            .get_last_instruction()
            .map(|instruction| instruction.get_opcode())
        {
            Some(inkwell::values::InstructionOpcode::Br) => {}
            Some(inkwell::values::InstructionOpcode::Switch) => {}
            _ => context
                .build_unconditional_branch(context.current_function().borrow().return_block())?,
        }

        context.set_basic_block(context.current_function().borrow().return_block());
        context.build_return(None)?;

        Ok(())
    }
}
