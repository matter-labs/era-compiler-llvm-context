//!
//! The runtime code function.
//!

use std::marker::PhantomData;

use crate::context::address_space::IAddressSpace;
use crate::context::code_type::CodeType;
use crate::context::IContext;
use crate::eravm::context::function::runtime::Runtime;
use crate::eravm::context::Context;
use crate::eravm::Dependency;
use crate::eravm::WriteLLVM;

///
/// The runtime code function.
///
/// Is a special function that is only used by the front-end generated code.
///
#[derive(Debug)]
pub struct RuntimeCode<B, AS, D>
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

impl<B, AS, D> RuntimeCode<B, AS, D>
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

impl<B, AS, D> WriteLLVM<D> for RuntimeCode<B, AS, D>
where
    B: WriteLLVM<D>,
    AS: IAddressSpace + Clone + Copy + PartialEq + Eq + Into<inkwell::AddressSpace>,
    D: Dependency + Clone,
{
    fn declare(&mut self, context: &mut Context<D>) -> anyhow::Result<()> {
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

    fn into_llvm(self, context: &mut Context<D>) -> anyhow::Result<()> {
        context.set_current_function(Runtime::FUNCTION_RUNTIME_CODE)?;

        context.set_basic_block(context.current_function().borrow().entry_block());
        context.set_code_type(CodeType::Runtime);
        self.inner.into_llvm(context)?;
        match context
            .basic_block()
            .get_last_instruction()
            .map(|instruction| instruction.get_opcode())
        {
            Some(inkwell::values::InstructionOpcode::Br) => {}
            Some(inkwell::values::InstructionOpcode::Switch) => {}
            _ => context
                .build_unconditional_branch(context.current_function().borrow().return_block()),
        }

        context.set_basic_block(context.current_function().borrow().return_block());
        context.build_return(None);

        Ok(())
    }
}
