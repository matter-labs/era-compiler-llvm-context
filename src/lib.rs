//!
//! The LLVM context library.
//!

pub(crate) mod context;
pub(crate) mod debug_config;
pub(crate) mod eravm;
pub(crate) mod evm;
pub(crate) mod optimizer;
pub(crate) mod target_machine;

pub use self::context::argument::Argument;
pub use self::context::attribute::Attribute;
pub use self::context::block_key::BlockKey;
pub use self::context::code_type::CodeType;
pub use self::context::r#loop::Loop;
pub use self::context::IContext;
pub use self::debug_config::ir_type::IRType as DebugConfigIR;
pub use self::debug_config::DebugConfig;
pub use self::eravm::build_assembly_text as eravm_build_assembly_text;
pub use self::eravm::context::address_space::AddressSpace as EraVMAddressSpace;
pub use self::eravm::context::build::Build as EraVMBuild;
pub use self::eravm::context::evmla_data::EVMLAData as EraVMContextEVMLAData;
pub use self::eravm::context::function::block::evmla_data::EVMLAData as EraVMFunctionBlockEVMLAData;
pub use self::eravm::context::function::block::Block as EraVMFunctionBlock;
pub use self::eravm::context::function::declaration::Declaration as EraVMFunctionDeclaration;
pub use self::eravm::context::function::evmla_data::EVMLAData as EraVMFunctionEVMLAData;
pub use self::eravm::context::function::intrinsics::Intrinsics as EraVMIntrinsicFunction;
pub use self::eravm::context::function::llvm_runtime::LLVMRuntime as EraVMLLVMRuntime;
pub use self::eravm::context::function::r#return::Return as EraVMFunctionReturn;
pub use self::eravm::context::function::runtime::deploy_code::DeployCode as EraVMDeployCodeFunction;
pub use self::eravm::context::function::runtime::entry::Entry as EraVMEntryFunction;
pub use self::eravm::context::function::runtime::runtime_code::RuntimeCode as EraVMRuntimeCodeFunction;
pub use self::eravm::context::function::runtime::Runtime as EraVMRuntime;
pub use self::eravm::context::function::vyper_data::VyperData as EraVMFunctionVyperData;
pub use self::eravm::context::function::yul_data::YulData as EraVMFunctionYulData;
pub use self::eravm::context::function::Function as EraVMFunction;
pub use self::eravm::context::global::Global as EraVMGlobal;
pub use self::eravm::context::pointer::Pointer as EraVMPointer;
pub use self::eravm::context::solidity_data::SolidityData as EraVMContextSolidityData;
pub use self::eravm::context::vyper_data::VyperData as EraVMContextVyperData;
pub use self::eravm::context::yul_data::YulData as EraVMContextYulData;
pub use self::eravm::context::Context as EraVMContext;
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
pub use self::eravm::metadata_hash::MetadataHash as EraVMMetadataHash;
pub use self::eravm::r#const as eravm_const;
pub use self::eravm::utils as eravm_utils;
pub use self::eravm::Dependency as EraVMDependency;
pub use self::eravm::DummyDependency as EraVMDummyDependency;
pub use self::eravm::DummyLLVMWritable as EraVMDummyLLVMWritable;
pub use self::eravm::WriteLLVM as EraVMWriteLLVM;
pub use self::evm::context::address_space::AddressSpace as EVMAddressSpace;
pub use self::evm::context::build::Build as EVMBuild;
pub use self::evm::context::evmla_data::EVMLAData as EVMContextEVMLAData;
pub use self::evm::context::function::block::evmla_data::EVMLAData as EVMFunctionBlockEVMLAData;
pub use self::evm::context::function::block::Block as EVMFunctionBlock;
pub use self::evm::context::function::declaration::Declaration as EVMFunctionDeclaration;
pub use self::evm::context::function::evmla_data::EVMLAData as EVMFunctionEVMLAData;
pub use self::evm::context::function::intrinsics::Intrinsics as EVMIntrinsicFunction;
pub use self::evm::context::function::r#return::Return as EVMFunctionReturn;
pub use self::evm::context::function::runtime::entry::Entry as EVMEntryFunction;
pub use self::evm::context::function::vyper_data::VyperData as EVMFunctionVyperData;
pub use self::evm::context::function::Function as EVMFunction;
pub use self::evm::context::pointer::Pointer as EVMPointer;
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
pub use self::evm::metadata_hash::MetadataHash as EVMMetadataHash;
pub use self::evm::r#const as evm_const;
pub use self::evm::utils as evm_utils;
pub use self::evm::Dependency as EVMDependency;
pub use self::evm::DummyDependency as EVMDummyDependency;
pub use self::evm::DummyLLVMWritable as EVMDummyLLVMWritable;
pub use self::evm::WriteLLVM as EVMWriteLLVM;
pub use self::optimizer::settings::size_level::SizeLevel as OptimizerSettingsSizeLevel;
pub use self::optimizer::settings::Settings as OptimizerSettings;
pub use self::optimizer::Optimizer;
pub use self::target_machine::target::Target;
pub use self::target_machine::TargetMachine;

///
/// Initializes the target machine.
///
pub fn initialize_target(target: Target) {
    match target {
        Target::EraVM => self::eravm::initialize_target(),
        Target::EVM => self::evm::initialize_target(),
    }
}
