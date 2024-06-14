//!
//! The LLVM context library.
//!

pub mod r#const;
pub mod context;
pub mod evm;
pub mod extensions;
pub mod metadata_hash;
pub mod utils;

pub use self::r#const::*;

use crate::debug_config::DebugConfig;
use crate::dependency::Dependency;

use self::context::build::Build;
use self::context::Context;

///
/// Initializes the EraVM target machine.
///
pub fn initialize_target() {
    inkwell::targets::Target::initialize_eravm(&inkwell::targets::InitializationConfig::default());
}

///
/// Encodes EraVM assembly into bytecode.
///
pub fn from_assembly(
    contract_path: &str,
    assembly_text: String,
    metadata_hash: Option<[u8; era_compiler_common::BYTE_LENGTH_FIELD]>,
    output_assembly: bool,
    debug_config: Option<&DebugConfig>,
) -> anyhow::Result<Build> {
    if let Some(debug_config) = debug_config {
        debug_config.dump_assembly(contract_path, None, assembly_text.as_str())?;
    }

    let output_assembly = if output_assembly {
        Some(assembly_text.clone())
    } else {
        None
    };

    let mut assembly = zkevm_assembly::Assembly::from_string(assembly_text, metadata_hash)
        .map_err(|error| anyhow::anyhow!("assembly parsing: {error}"))?;

    let bytecode_words = match zkevm_assembly::get_encoding_mode() {
        zkevm_assembly::RunningVmEncodingMode::Production => { assembly.compile_to_bytecode_for_mode::<8, zkevm_opcode_defs::decoding::EncodingModeProduction>() },
        zkevm_assembly::RunningVmEncodingMode::Testing => { assembly.compile_to_bytecode_for_mode::<16, zkevm_opcode_defs::decoding::EncodingModeTesting>() },
    }
        .map_err(|error| {
            anyhow::anyhow!(
                "assembly-to-bytecode conversion: {error}",
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
    .map_err(|_error| anyhow::anyhow!("bytecode hashing"))?;

    let bytecode = bytecode_words.into_iter().flatten().collect();

    Ok(Build::new(
        bytecode,
        bytecode_hash,
        metadata_hash,
        output_assembly,
    ))
}

///
/// Implemented by items which are translated into LLVM IR.
///
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
#[derive(Debug, Default, Clone)]
pub struct DummyLLVMWritable {}

impl<D> WriteLLVM<D> for DummyLLVMWritable
where
    D: Dependency,
{
    fn into_llvm(self, _context: &mut Context<D>) -> anyhow::Result<()> {
        Ok(())
    }
}
