//!
//! The entry function.
//!

use crate::context::IContext;
use crate::evm::attribute::Attribute;
use crate::evm::context::Context;
use crate::evm::WriteLLVM;

///
/// The entry function.
///
/// Is a special runtime function that is only used by the front-end generated code.
///
#[derive(Debug)]
pub struct Entry<B>
where
    B: WriteLLVM,
{
    /// The runtime code AST representation.
    inner: B,
}

impl<B> Entry<B>
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

impl<B> WriteLLVM for Entry<B>
where
    B: WriteLLVM,
{
    fn declare(&mut self, context: &mut Context) -> anyhow::Result<()> {
        let function_type = context.function_type::<inkwell::types::BasicTypeEnum>(vec![], 0);
        let function = context.add_function(
            crate::r#const::ENTRY_FUNCTION_NAME,
            function_type,
            0,
            Some(inkwell::module::Linkage::External),
        )?;
        function.borrow().declaration().value.add_attribute(
            inkwell::attributes::AttributeLoc::Function,
            context
                .llvm()
                .create_string_attribute(Attribute::EVMEntryFunction.to_string().as_str(), ""),
        );

        self.inner.declare(context)
    }

    fn into_llvm(self, context: &mut Context) -> anyhow::Result<()> {
        context.set_current_function(crate::r#const::ENTRY_FUNCTION_NAME)?;

        context.set_basic_block(context.current_function().borrow().entry_block());
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
        crate::evm::instructions::r#return::stop(context)?;

        Ok(())
    }
}
