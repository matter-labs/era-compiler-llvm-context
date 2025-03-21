//!
//! The runtime code function.
//!

use crate::context::IContext;
use crate::eravm::context::function::runtime::Runtime;
use crate::eravm::context::Context;
use crate::eravm::WriteLLVM;

///
/// The runtime code function.
///
/// Is a special function that is only used by the front-end generated code.
///
#[derive(Debug)]
pub struct RuntimeCode<B>
where
    B: WriteLLVM,
{
    /// The runtime code AST representation.
    inner: B,
}

impl<B> RuntimeCode<B>
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

impl<B> WriteLLVM for RuntimeCode<B>
where
    B: WriteLLVM,
{
    fn declare(&mut self, context: &mut Context) -> anyhow::Result<()> {
        let function_type =
            context.function_type::<inkwell::types::BasicTypeEnum>(vec![], 0, false);
        context.add_function(
            Runtime::FUNCTION_RUNTIME_CODE,
            function_type,
            0,
            Some(inkwell::module::Linkage::Private),
        )?;

        self.inner.declare(context)
    }

    fn into_llvm(self, context: &mut Context) -> anyhow::Result<()> {
        context.set_current_function(Runtime::FUNCTION_RUNTIME_CODE)?;

        context.set_basic_block(context.current_function().borrow().entry_block());
        context.set_code_segment(era_compiler_common::CodeSegment::Runtime);
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
