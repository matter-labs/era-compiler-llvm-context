//!
//! The entry function.
//!

use std::marker::PhantomData;

use crate::context::address_space::IAddressSpace;
use crate::context::IContext;
use crate::evm::context::Context;
use crate::evm::Dependency;
use crate::evm::WriteLLVM;

///
/// The entry function.
///
/// Is a special runtime function that is only used by the front-end generated code.
///
#[derive(Debug)]
pub struct Entry<B, AS, D>
where
    B: WriteLLVM<D>,
    AS: IAddressSpace + Clone + Copy + PartialEq + Eq + Into<inkwell::AddressSpace>,
    D: Dependency + Clone,
{
    /// The runtime code AST representation.
    inner: B,
    /// The `D` phantom data.
    _pd_d: PhantomData<D>,
    /// The `D` phantom data.
    _pd_as: PhantomData<AS>,
}

impl<B, AS, D> Entry<B, AS, D>
where
    B: WriteLLVM<D>,
    AS: IAddressSpace + Clone + Copy + PartialEq + Eq + Into<inkwell::AddressSpace>,
    D: Dependency + Clone,
{
    ///
    /// A shortcut constructor.
    ///
    pub fn new(inner: B) -> Self {
        Self {
            inner,
            _pd_d: PhantomData,
            _pd_as: PhantomData,
        }
    }
}

impl<B, AS, D> WriteLLVM<D> for Entry<B, AS, D>
where
    B: WriteLLVM<D>,
    AS: IAddressSpace + Clone + Copy + PartialEq + Eq + Into<inkwell::AddressSpace>,
    D: Dependency + Clone,
{
    fn declare(&mut self, context: &mut Context<D>) -> anyhow::Result<()> {
        let function_type = context.function_type::<inkwell::types::BasicTypeEnum>(vec![], 0);
        context.add_function(
            crate::evm::r#const::ENTRY_FUNCTION_NAME,
            function_type,
            0,
            Some(inkwell::module::Linkage::External),
        )?;

        self.inner.declare(context)
    }

    fn into_llvm(self, context: &mut Context<D>) -> anyhow::Result<()> {
        context.set_current_function(crate::evm::r#const::ENTRY_FUNCTION_NAME)?;

        context.set_basic_block(context.current_function().borrow().entry_block());
        self.inner.into_llvm(context)?;
        match context
            .basic_block()
            .get_last_instruction()
            .map(|instruction| instruction.get_opcode())
        {
            Some(inkwell::values::InstructionOpcode::Br) => {}
            Some(inkwell::values::InstructionOpcode::Switch) => {}
            _ => context.build_unreachable(),
        }

        context.set_basic_block(context.current_function().borrow().return_block());
        context.build_return(None);

        Ok(())
    }
}
