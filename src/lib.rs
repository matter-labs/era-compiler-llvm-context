//!
//! The LLVM context library.
//!

pub(crate) mod r#const;
pub(crate) mod context;
pub(crate) mod debug_config;
pub(crate) mod evm;
pub(crate) mod metadata_hash;
pub(crate) mod utils;
pub(crate) mod zkevm;

pub use self::context::address_space::AddressSpace;
pub use self::context::argument::Argument;
pub use self::context::attribute::Attribute;
pub use self::context::build::Build;
pub use self::context::code_type::CodeType;
pub use self::context::evmla_data::EVMLAData as ContextEVMLAData;
pub use self::context::function::block::evmla_data::key::Key as FunctionBlockKey;
pub use self::context::function::block::evmla_data::EVMLAData as FunctionBlockEVMLAData;
pub use self::context::function::block::Block as FunctionBlock;
pub use self::context::function::declaration::Declaration as FunctionDeclaration;
pub use self::context::function::evmla_data::EVMLAData as FunctionEVMLAData;
pub use self::context::function::intrinsics::Intrinsics as IntrinsicFunction;
pub use self::context::function::llvm_runtime::LLVMRuntime;
pub use self::context::function::r#return::Return as FunctionReturn;
pub use self::context::function::runtime::deploy_code::DeployCode as DeployCodeFunction;
pub use self::context::function::runtime::entry::Entry as EntryFunction;
pub use self::context::function::runtime::runtime_code::RuntimeCode as RuntimeCodeFunction;
pub use self::context::function::runtime::Runtime;
pub use self::context::function::vyper_data::VyperData as FunctionVyperData;
pub use self::context::function::yul_data::YulData as FunctionYulData;
pub use self::context::function::Function;
pub use self::context::global::Global;
pub use self::context::optimizer::settings::size_level::SizeLevel as OptimizerSettingsSizeLevel;
pub use self::context::optimizer::settings::Settings as OptimizerSettings;
pub use self::context::optimizer::Optimizer;
pub use self::context::pointer::Pointer;
pub use self::context::r#loop::Loop;
pub use self::context::solidity_data::SolidityData as ContextSolidityData;
pub use self::context::target_machine::TargetMachine;
pub use self::context::vyper_data::VyperData as ContextVyperData;
pub use self::context::yul_data::YulData as ContextYulData;
pub use self::context::Context;
pub use self::debug_config::ir_type::IRType as DebugConfigIR;
pub use self::debug_config::DebugConfig;
pub use self::evm::arithmetic;
pub use self::evm::bitwise;
pub use self::evm::call;
pub use self::evm::calldata;
pub use self::evm::comparison;
pub use self::evm::context as contract_context;
pub use self::evm::create;
pub use self::evm::ether_gas;
pub use self::evm::event;
pub use self::evm::ext_code;
pub use self::evm::immutable;
pub use self::evm::math;
pub use self::evm::memory;
pub use self::evm::r#return;
pub use self::evm::return_data;
pub use self::evm::storage;
pub use self::metadata_hash::MetadataHash;
pub use self::r#const::*;
pub use self::utils::*;
pub use self::zkevm::abi as zkevm_abi;
pub use self::zkevm::call as zkevm_call;
pub use self::zkevm::general as zkevm_general;
pub use self::zkevm::math as zkevm_math;

use std::sync::Arc;
use std::sync::RwLock;

///
/// Initializes the zkEVM target machine.
///
pub fn initialize_target() {
    inkwell::targets::Target::initialize_syncvm(&inkwell::targets::InitializationConfig::default());
}

///
/// Builds zkEVM assembly text.
///
pub fn build_assembly_text(
    contract_path: &str,
    assembly_text: &str,
    metadata_hash: Option<[u8; compiler_common::BYTE_LENGTH_FIELD]>,
    debug_config: Option<&DebugConfig>,
) -> anyhow::Result<Build> {
    if let Some(debug_config) = debug_config {
        debug_config.dump_assembly(contract_path, assembly_text)?;
    }

    let assembly = zkevm_assembly::Assembly::from_string(assembly_text.to_owned(), metadata_hash)
        .map_err(|error| {
        anyhow::anyhow!(
            "The contract `{}` assembly parsing error: {}",
            contract_path,
            error,
        )
    })?;

    let bytecode_words = match zkevm_assembly::get_encoding_mode() {
        zkevm_assembly::RunningVmEncodingMode::Production => { assembly.clone().compile_to_bytecode_for_mode::<8, zkevm_opcode_defs::decoding::EncodingModeProduction>() },
        zkevm_assembly::RunningVmEncodingMode::Testing => { assembly.clone().compile_to_bytecode_for_mode::<16, zkevm_opcode_defs::decoding::EncodingModeTesting>() },
    }
        .map_err(|error| {
            anyhow::anyhow!(
                "The contract `{}` assembly-to-bytecode conversion error: {}",
                contract_path,
                error,
            )
        })?;

    let bytecode_hash = match zkevm_assembly::get_encoding_mode() {
        zkevm_assembly::RunningVmEncodingMode::Production => {
            zkevm_opcode_defs::utils::bytecode_to_code_hash_for_mode::<
                8,
                zkevm_opcode_defs::decoding::EncodingModeProduction,
            >(bytecode_words.as_slice())
        }
        zkevm_assembly::RunningVmEncodingMode::Testing => {
            zkevm_opcode_defs::utils::bytecode_to_code_hash_for_mode::<
                16,
                zkevm_opcode_defs::decoding::EncodingModeTesting,
            >(bytecode_words.as_slice())
        }
    }
    .map(hex::encode)
    .map_err(|_error| {
        anyhow::anyhow!("The contract `{}` bytecode hashing error", contract_path,)
    })?;

    let bytecode = bytecode_words.into_iter().flatten().collect();

    Ok(Build::new(
        assembly_text.to_owned(),
        assembly,
        bytecode,
        bytecode_hash,
    ))
}

///
/// Implemented by items which are translated into LLVM IR.
///
#[allow(clippy::upper_case_acronyms)]
pub trait WriteLLVM<D>
where
    D: Dependency,
{
    ///
    /// Declares the entity in the LLVM IR.
    /// Is usually performed in order to use the item before defining it.
    ///
    fn declare(&mut self, _context: &mut Context<D>) -> anyhow::Result<()> {
        Ok(())
    }

    ///
    /// Translates the entity into LLVM IR.
    ///
    fn into_llvm(self, context: &mut Context<D>) -> anyhow::Result<()>;
}

///
/// The dummy LLVM writable entity.
///
#[derive(Debug, Default)]
pub struct DummyLLVMWritable {}

impl<D> WriteLLVM<D> for DummyLLVMWritable
where
    D: Dependency,
{
    fn into_llvm(self, _context: &mut Context<D>) -> anyhow::Result<()> {
        Ok(())
    }
}

///
/// Implemented by items managing project dependencies.
///
pub trait Dependency {
    ///
    /// Compiles a project dependency.
    ///
    fn compile(
        dependency: Arc<RwLock<Self>>,
        path: &str,
        target_machine: TargetMachine,
        optimizer_settings: OptimizerSettings,
        is_system_mode: bool,
        include_metadata_hash: bool,
        debug_config: Option<DebugConfig>,
    ) -> anyhow::Result<String>;

    ///
    /// Resolves a full contract path.
    ///
    fn resolve_path(&self, identifier: &str) -> anyhow::Result<String>;

    ///
    /// Resolves a library address.
    ///
    fn resolve_library(&self, path: &str) -> anyhow::Result<String>;
}

///
/// The dummy dependency entity.
///
#[derive(Debug, Default)]
pub struct DummyDependency {}

impl Dependency for DummyDependency {
    fn compile(
        _dependency: Arc<RwLock<Self>>,
        _path: &str,
        _target_machine: TargetMachine,
        _optimizer_settings: OptimizerSettings,
        _is_system_mode: bool,
        _include_metadata_hash: bool,
        _debug_config: Option<DebugConfig>,
    ) -> anyhow::Result<String> {
        Ok(String::new())
    }

    ///
    /// Resolves a full contract path.
    ///
    fn resolve_path(&self, _identifier: &str) -> anyhow::Result<String> {
        Ok(String::new())
    }

    ///
    /// Resolves a library address.
    ///
    fn resolve_library(&self, _path: &str) -> anyhow::Result<String> {
        Ok(String::new())
    }
}
