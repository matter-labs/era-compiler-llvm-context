//!
//! The LLVM context library.
//!

pub mod r#const;
pub mod context;
pub mod evm;
pub mod extensions;
pub mod utils;

pub use self::r#const::*;

use crate::debug_config::DebugConfig;
use crate::dependency::Dependency;
use crate::eravm::context::build::Build;
use crate::target_machine::TargetMachine;

use self::context::Context;

///
/// Initializes the EraVM target machine.
///
pub fn initialize_target() {
    inkwell::targets::Target::initialize_eravm(&inkwell::targets::InitializationConfig::default());
}

///
/// Translates `assembly_text` to an object code.
///
pub fn assemble(
    target_machine: &TargetMachine,
    contract_path: &str,
    assembly_text: &str,
    debug_config: Option<&DebugConfig>,
) -> anyhow::Result<inkwell::memory_buffer::MemoryBuffer> {
    if let Some(debug_config) = debug_config {
        debug_config.dump_assembly(contract_path, None, assembly_text)?;
    }

    let assembly_buffer = inkwell::memory_buffer::MemoryBuffer::create_from_memory_range(
        assembly_text.as_bytes(),
        "assembly_buffer",
        false,
    );

    let bytecode_buffer = target_machine
        .assemble(&assembly_buffer)
        .map_err(|error| anyhow::anyhow!("assembling: {error}"))?;
    Ok(bytecode_buffer)
}

///
/// Disassembles `bytecode`, returning textual representation.
///
pub fn disassemble(target_machine: &TargetMachine, bytecode: &[u8]) -> anyhow::Result<String> {
    let bytecode_buffer = inkwell::memory_buffer::MemoryBuffer::create_from_memory_range(
        bytecode,
        "bytecode_buffer",
        false,
    );

    let disassembly_buffer = target_machine
        .disassemble(&bytecode_buffer, 0, DISASSEMBLER_DEFAULT_MODE)
        .map_err(|error| anyhow::anyhow!("disassembling: {error}"))?;

    let disassembly_text = String::from_utf8_lossy(disassembly_buffer.as_slice());
    Ok(disassembly_text.to_string())
}

///
/// Converts `bytecode_buffer` and auxiliary data into a build.
///
pub fn build(
    bytecode_buffer: inkwell::memory_buffer::MemoryBuffer,
    linker_symbols: &[(
        [u8; era_compiler_common::BYTE_LENGTH_FIELD],
        [u8; era_compiler_common::BYTE_LENGTH_ETH_ADDRESS],
    )],
    metadata_hash: Option<era_compiler_common::Hash>,
    assembly_text: Option<String>,
) -> anyhow::Result<Build> {
    let metadata_hash = metadata_hash.as_ref().map(|hash| match hash {
        era_compiler_common::Hash::Keccak256 { bytes, .. } => bytes.to_vec(),
        hash @ era_compiler_common::Hash::Ipfs { .. } => hash.as_cbor_bytes(),
    });
    let bytecode_buffer_with_metadata = match metadata_hash {
        Some(ref metadata) => bytecode_buffer
            .append_metadata_eravm(metadata.as_slice())
            .map_err(|error| anyhow::anyhow!("bytecode metadata appending error: {error}"))?,
        None => bytecode_buffer,
    };
    let bytecode_buffer_linked = bytecode_buffer_with_metadata
        .link_module_eravm(linker_symbols)
        .map_err(|error| anyhow::anyhow!("bytecode linking error: {error}"))?;
    let bytecode = bytecode_buffer_linked.as_slice().to_vec();

    let bytecode_words: Vec<[u8; era_compiler_common::BYTE_LENGTH_FIELD]> = bytecode
        .chunks(era_compiler_common::BYTE_LENGTH_FIELD)
        .map(|word| word.try_into().expect("Always valid"))
        .collect();
    let bytecode_hash = zkevm_opcode_defs::utils::bytecode_to_code_hash_for_mode::<
        { era_compiler_common::BYTE_LENGTH_X64 },
        zkevm_opcode_defs::decoding::EncodingModeProduction,
    >(bytecode_words.as_slice())
    .map_err(|_| anyhow::anyhow!("bytecode hashing error"))?;

    let build = Build::new(bytecode, bytecode_hash, metadata_hash, assembly_text);
    Ok(build)
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
