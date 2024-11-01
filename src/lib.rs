//!
//! The LLVM context library.
//!

#![allow(clippy::too_many_arguments)]
#![allow(clippy::upper_case_acronyms)]

pub(crate) mod r#const;
pub(crate) mod context;
pub(crate) mod debug_config;
pub(crate) mod debug_info;
pub(crate) mod dependency;
pub(crate) mod eravm;
pub(crate) mod evm;
pub(crate) mod optimizer;
pub(crate) mod target_machine;

pub use self::context::attribute::memory::Memory as MemoryAttribute;
pub use self::context::attribute::Attribute;
pub use self::context::function::block::evmla_data::EVMLAData as FunctionBlockEVMLAData;
pub use self::context::function::block::key::Key as BlockKey;
pub use self::context::function::block::Block as FunctionBlock;
pub use self::context::function::declaration::Declaration as FunctionDeclaration;
pub use self::context::function::evmla_data::EVMLAData as FunctionEVMLAData;
pub use self::context::function::r#return::Return as FunctionReturn;
pub use self::context::pointer::Pointer;
pub use self::context::r#loop::Loop;
pub use self::context::traits::address_space::IAddressSpace;
pub use self::context::traits::evmla_data::IEVMLAData;
pub use self::context::traits::evmla_function::IEVMLAFunction;
pub use self::context::value::Value;
pub use self::context::IContext;
pub use self::debug_config::ir_type::IRType as DebugConfigIR;
pub use self::debug_config::DebugConfig;
pub use self::debug_info::DebugInfo;
pub use self::dependency::Dependency;
pub use self::dependency::DummyDependency;
pub use self::eravm::assemble as eravm_assemble;
pub use self::eravm::build as eravm_build;
pub use self::eravm::context::address_space::AddressSpace as EraVMAddressSpace;
pub use self::eravm::context::build::Build as EraVMBuild;
pub use self::eravm::context::evmla_data::EVMLAData as EraVMContextEVMLAData;
pub use self::eravm::context::function::intrinsics::Intrinsics as EraVMIntrinsicFunction;
pub use self::eravm::context::function::llvm_runtime::LLVMRuntime as EraVMLLVMRuntime;
pub use self::eravm::context::function::runtime::deploy_code::DeployCode as EraVMDeployCodeFunction;
pub use self::eravm::context::function::runtime::entry::Entry as EraVMEntryFunction;
pub use self::eravm::context::function::runtime::runtime_code::RuntimeCode as EraVMRuntimeCodeFunction;
pub use self::eravm::context::function::runtime::Runtime as EraVMRuntime;
pub use self::eravm::context::function::vyper_data::VyperData as EraVMFunctionVyperData;
pub use self::eravm::context::function::yul_data::YulData as EraVMFunctionYulData;
pub use self::eravm::context::function::Function as EraVMFunction;
pub use self::eravm::context::global::Global as EraVMGlobal;
pub use self::eravm::context::solidity_data::SolidityData as EraVMContextSolidityData;
pub use self::eravm::context::vyper_data::VyperData as EraVMContextVyperData;
pub use self::eravm::context::yul_data::YulData as EraVMContextYulData;
pub use self::eravm::context::Context as EraVMContext;
pub use self::eravm::disassemble as eravm_disassemble;
pub use self::eravm::evm::arithmetic as eravm_evm_arithmetic;
pub use self::eravm::evm::bitwise as eravm_evm_bitwise;
pub use self::eravm::evm::call as eravm_evm_call;
pub use self::eravm::evm::calldata as eravm_evm_calldata;
pub use self::eravm::evm::comparison as eravm_evm_comparison;
pub use self::eravm::evm::context as eravm_evm_contract_context;
pub use self::eravm::evm::create as eravm_evm_create;
pub use self::eravm::evm::crypto as eravm_evm_crypto;
pub use self::eravm::evm::ether_gas as eravm_evm_ether_gas;
pub use self::eravm::evm::event as eravm_evm_event;
pub use self::eravm::evm::ext_code as eravm_evm_ext_code;
pub use self::eravm::evm::immutable as eravm_evm_immutable;
pub use self::eravm::evm::math as eravm_evm_math;
pub use self::eravm::evm::memory as eravm_evm_memory;
pub use self::eravm::evm::r#return as eravm_evm_return;
pub use self::eravm::evm::return_data as eravm_evm_return_data;
pub use self::eravm::evm::storage as eravm_evm_storage;
pub use self::eravm::extensions::abi as eravm_abi;
pub use self::eravm::extensions::call as eravm_call;
pub use self::eravm::extensions::general as eravm_general;
pub use self::eravm::extensions::math as eravm_math;
pub use self::eravm::link as eravm_link;
pub use self::eravm::r#const as eravm_const;
pub use self::eravm::utils as eravm_utils;
pub use self::eravm::DummyLLVMWritable as EraVMDummyLLVMWritable;
pub use self::eravm::WriteLLVM as EraVMWriteLLVM;
pub use self::evm::context::address_space::AddressSpace as EVMAddressSpace;
pub use self::evm::context::build::Build as EVMBuild;
pub use self::evm::context::evmla_data::EVMLAData as EVMContextEVMLAData;
pub use self::evm::context::function::intrinsics::Intrinsics as EVMIntrinsicFunction;
pub use self::evm::context::function::runtime::entry::Entry as EVMEntryFunction;
pub use self::evm::context::function::vyper_data::VyperData as EVMFunctionVyperData;
pub use self::evm::context::function::Function as EVMFunction;
pub use self::evm::context::Context as EVMContext;
pub use self::evm::instructions::arithmetic as evm_arithmetic;
pub use self::evm::instructions::bitwise as evm_bitwise;
pub use self::evm::instructions::call as evm_call;
pub use self::evm::instructions::calldata as evm_calldata;
pub use self::evm::instructions::code as evm_code;
pub use self::evm::instructions::comparison as evm_comparison;
pub use self::evm::instructions::context as evm_contract_context;
pub use self::evm::instructions::create as evm_create;
pub use self::evm::instructions::ether_gas as evm_ether_gas;
pub use self::evm::instructions::event as evm_event;
pub use self::evm::instructions::immutable as evm_immutable;
pub use self::evm::instructions::math as evm_math;
pub use self::evm::instructions::memory as evm_memory;
pub use self::evm::instructions::r#return as evm_return;
pub use self::evm::instructions::return_data as evm_return_data;
pub use self::evm::instructions::storage as evm_storage;
pub use self::evm::r#const as evm_const;
pub use self::evm::DummyLLVMWritable as EVMDummyLLVMWritable;
pub use self::evm::WriteLLVM as EVMWriteLLVM;
pub use self::optimizer::settings::size_level::SizeLevel as OptimizerSettingsSizeLevel;
pub use self::optimizer::settings::Settings as OptimizerSettings;
pub use self::optimizer::Optimizer;
pub use self::r#const::*;
pub use self::target_machine::TargetMachine;

///
/// Initializes the target machine.
///
pub fn initialize_target(target: era_compiler_common::Target) {
    match target {
        era_compiler_common::Target::EraVM => self::eravm::initialize_target(),
        era_compiler_common::Target::EVM => self::evm::initialize_target(),
    }
}
