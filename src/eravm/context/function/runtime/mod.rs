//!
//! The front-end runtime functions.
//!

pub mod default_call;
pub mod deploy_code;
pub mod deployer_call;
pub mod entry;
pub mod runtime_code;

use crate::context::function::declaration::Declaration as FunctionDeclaration;
use crate::context::IContext;
use crate::eravm::context::address_space::AddressSpace;
use crate::eravm::context::Context;
use crate::eravm::WriteLLVM;

use self::default_call::DefaultCall;
use self::deployer_call::DeployerCall;

///
/// The front-end runtime functions.
///
#[derive(Debug, Default, Clone)]
pub struct Runtime {}

impl Runtime {
    /// The main entry function name.
    pub const FUNCTION_ENTRY: &'static str = "__entry";

    /// The deploy code function name.
    pub const FUNCTION_DEPLOY_CODE: &'static str = "__deploy";

    /// The runtime code function name.
    pub const FUNCTION_RUNTIME_CODE: &'static str = "__runtime";

    ///
    /// Returns the corresponding runtime function.
    ///
    pub fn default_call<'ctx>(
        context: &Context<'ctx>,
        call_function: FunctionDeclaration<'ctx>,
    ) -> FunctionDeclaration<'ctx> {
        context
            .get_function(DefaultCall::name(call_function).as_str())
            .expect("Always exists")
            .borrow()
            .declaration()
    }

    ///
    /// Returns the corresponding runtime function.
    ///
    pub fn deployer_call<'ctx>(
        context: &Context<'ctx>,
        address_space: AddressSpace,
    ) -> FunctionDeclaration<'ctx> {
        context
            .get_function(DeployerCall::name(address_space).as_str())
            .expect("Always exists")
            .borrow()
            .declaration()
    }
}

impl WriteLLVM for Runtime {
    fn declare(&mut self, context: &mut Context) -> anyhow::Result<()> {
        DefaultCall::new(context.llvm_runtime().far_call).declare(context)?;
        DefaultCall::new(context.llvm_runtime().static_call).declare(context)?;
        DefaultCall::new(context.llvm_runtime().delegate_call).declare(context)?;
        DeployerCall::new(AddressSpace::Heap).declare(context)?;
        DeployerCall::new(AddressSpace::HeapAuxiliary).declare(context)?;

        Ok(())
    }

    fn into_llvm(self, context: &mut Context) -> anyhow::Result<()> {
        DefaultCall::new(context.llvm_runtime().far_call).into_llvm(context)?;
        DefaultCall::new(context.llvm_runtime().static_call).into_llvm(context)?;
        DefaultCall::new(context.llvm_runtime().delegate_call).into_llvm(context)?;
        DeployerCall::new(AddressSpace::Heap).into_llvm(context)?;
        DeployerCall::new(AddressSpace::HeapAuxiliary).into_llvm(context)?;

        Ok(())
    }
}
