//!
//! The front-end runtime functions.
//!

pub mod default_call;
pub mod deploy_code;
pub mod deployer_call;
pub mod entry;
pub mod exit;
pub mod exponent;
pub mod keccak256;
pub mod runtime_code;
pub mod system_request;

use crate::eravm::context::address_space::AddressSpace;
use crate::eravm::context::function::declaration::Declaration as FunctionDeclaration;
use crate::eravm::context::Context;
use crate::eravm::Dependency;
use crate::eravm::WriteLLVM;

use self::default_call::DefaultCall;
use self::deployer_call::DeployerCall;
use self::exit::Exit;
use self::exponent::Exponent;
use self::keccak256::Keccak256;
use self::system_request::SystemRequest;

///
/// The front-end runtime functions.
///
#[derive(Debug, Clone)]
pub struct Runtime {
    /// The address space where the calldata is allocated.
    /// Solidity uses the ordinary heap. Vyper uses the auxiliary heap.
    address_space: AddressSpace,
}

impl Runtime {
    /// The main entry function name.
    pub const FUNCTION_ENTRY: &'static str = "__entry";

    /// The deploy code function name.
    pub const FUNCTION_DEPLOY_CODE: &'static str = "__deploy";

    /// The runtime code function name.
    pub const FUNCTION_RUNTIME_CODE: &'static str = "__runtime";

    ///
    /// A shortcut constructor.
    ///
    pub fn new(address_space: AddressSpace) -> Self {
        Self { address_space }
    }

    ///
    /// Returns the corresponding runtime function.
    ///
    pub fn exponent<'ctx, D>(context: &Context<'ctx, D>) -> FunctionDeclaration<'ctx>
    where
        D: Dependency + Clone,
    {
        context
            .get_function(Exponent::FUNCTION_NAME)
            .expect("Always exists")
            .borrow()
            .declaration()
    }

    ///
    /// Returns the corresponding runtime function.
    ///
    pub fn default_call<'ctx, D>(
        context: &Context<'ctx, D>,
        call_function: FunctionDeclaration<'ctx>,
    ) -> FunctionDeclaration<'ctx>
    where
        D: Dependency + Clone,
    {
        context
            .get_function(DefaultCall::name(call_function).as_str())
            .expect("Always exists")
            .borrow()
            .declaration()
    }

    ///
    /// Returns the corresponding runtime function.
    ///
    pub fn keccak256<'ctx, D>(context: &Context<'ctx, D>) -> FunctionDeclaration<'ctx>
    where
        D: Dependency + Clone,
    {
        context
            .get_function(Keccak256::FUNCTION_NAME)
            .expect("Always exists")
            .borrow()
            .declaration()
    }

    ///
    /// Returns the corresponding runtime function.
    ///
    pub fn system_request<'ctx, D>(context: &Context<'ctx, D>) -> FunctionDeclaration<'ctx>
    where
        D: Dependency + Clone,
    {
        context
            .get_function(SystemRequest::FUNCTION_NAME)
            .expect("Always exists")
            .borrow()
            .declaration()
    }

    ///
    /// Returns the corresponding runtime function.
    ///
    pub fn deployer_call<'ctx, D>(context: &Context<'ctx, D>) -> FunctionDeclaration<'ctx>
    where
        D: Dependency + Clone,
    {
        context
            .get_function(DeployerCall::FUNCTION_NAME)
            .expect("Always exists")
            .borrow()
            .declaration()
    }

    ///
    /// Returns the corresponding runtime function.
    ///
    pub fn exit<'ctx, D>(
        context: &Context<'ctx, D>,
        return_function: FunctionDeclaration<'ctx>,
    ) -> FunctionDeclaration<'ctx>
    where
        D: Dependency + Clone,
    {
        context
            .get_function(Exit::name(context, return_function).as_str())
            .expect("Always exists")
            .borrow()
            .declaration()
    }
}

impl<D> WriteLLVM<D> for Runtime
where
    D: Dependency + Clone,
{
    fn declare(&mut self, context: &mut Context<D>) -> anyhow::Result<()> {
        Exponent::default().declare(context)?;
        DefaultCall::new(context.llvm_runtime().far_call).declare(context)?;
        DefaultCall::new(context.llvm_runtime().static_call).declare(context)?;
        DefaultCall::new(context.llvm_runtime().delegate_call).declare(context)?;
        Keccak256::default().declare(context)?;
        SystemRequest::default().declare(context)?;
        DeployerCall::new(self.address_space).declare(context)?;
        Exit::new(context, context.intrinsics().r#return).declare(context)?;
        Exit::new(context, context.intrinsics().revert).declare(context)?;

        Ok(())
    }

    fn into_llvm(self, context: &mut Context<D>) -> anyhow::Result<()> {
        Exponent::default().into_llvm(context)?;
        DefaultCall::new(context.llvm_runtime().far_call).into_llvm(context)?;
        DefaultCall::new(context.llvm_runtime().static_call).into_llvm(context)?;
        DefaultCall::new(context.llvm_runtime().delegate_call).into_llvm(context)?;
        Keccak256::default().into_llvm(context)?;
        SystemRequest::default().into_llvm(context)?;
        DeployerCall::new(self.address_space).into_llvm(context)?;
        Exit::new(context, context.intrinsics().r#return).into_llvm(context)?;
        Exit::new(context, context.intrinsics().revert).into_llvm(context)?;

        Ok(())
    }
}
