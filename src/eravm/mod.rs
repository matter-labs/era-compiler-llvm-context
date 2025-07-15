//!
//! The LLVM EraVM context library.
//!

pub mod build;
pub mod r#const;
pub mod context;
pub mod evm;
pub mod extensions;
pub mod utils;

pub use self::r#const::*;

use std::collections::BTreeMap;

use crate::debug_config::DebugConfig;
use crate::target_machine::TargetMachine;

use self::build::Build;
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
        debug_config.dump_assembly(
            contract_path,
            era_compiler_common::Target::EraVM,
            assembly_text,
            false,
            None,
        )?;
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
pub fn disassemble(
    target_machine: &TargetMachine,
    bytecode_buffer: &inkwell::memory_buffer::MemoryBuffer,
) -> anyhow::Result<String> {
    let disassembly_buffer = target_machine
        .disassemble(bytecode_buffer, 0, DISASSEMBLER_DEFAULT_MODE)
        .map_err(|error| anyhow::anyhow!("disassembling: {error}"))?;

    let disassembly_text = String::from_utf8_lossy(disassembly_buffer.as_slice());
    Ok(disassembly_text.to_string())
}

///
/// Links `bytecode_buffer` with `linker_symbols` and `factory_dependencies`.
///
pub fn link(
    bytecode_buffer: inkwell::memory_buffer::MemoryBuffer,
    linker_symbols: &BTreeMap<String, [u8; era_compiler_common::BYTE_LENGTH_ETH_ADDRESS]>,
    factory_dependencies: &BTreeMap<String, [u8; era_compiler_common::BYTE_LENGTH_FIELD]>,
) -> anyhow::Result<(
    inkwell::memory_buffer::MemoryBuffer,
    era_compiler_common::ObjectFormat,
)> {
    if !bytecode_buffer.is_elf_eravm() {
        return Ok((bytecode_buffer, era_compiler_common::ObjectFormat::Raw));
    }

    let bytecode_buffer_linked = bytecode_buffer
        .link_eravm(linker_symbols, factory_dependencies)
        .map_err(|error| anyhow::anyhow!("linking: {error}"))?;
    let object_format = if bytecode_buffer_linked.is_elf_eravm() {
        era_compiler_common::ObjectFormat::ELF
    } else {
        era_compiler_common::ObjectFormat::Raw
    };
    Ok((bytecode_buffer_linked, object_format))
}

///
/// Computes the EraVM bytecode hash.
///
/// # Panics
/// If `bytecode_buffer` is an ELF object.
///
/// # Errors
/// If the bytecode size is not an odd number of 32-byte words.
///
pub fn hash(
    bytecode_buffer: &inkwell::memory_buffer::MemoryBuffer,
) -> anyhow::Result<[u8; era_compiler_common::BYTE_LENGTH_FIELD]> {
    assert!(
        !bytecode_buffer.is_elf_eravm(),
        "bytecode is still an unlinked ELF object"
    );

    let bytecode_words: Vec<[u8; era_compiler_common::BYTE_LENGTH_FIELD]> = bytecode_buffer
        .as_slice()
        .chunks(era_compiler_common::BYTE_LENGTH_FIELD)
        .map(|word| word.try_into().expect("Always valid"))
        .collect();
    let bytecode_hash = zkevm_opcode_defs::utils::bytecode_to_code_hash_for_mode::<
        { era_compiler_common::BYTE_LENGTH_X64 },
        zkevm_opcode_defs::decoding::EncodingModeProduction,
    >(bytecode_words.as_slice())
    .map_err(|_| anyhow::anyhow!("bytecode hashing error"))?;
    Ok(bytecode_hash)
}

///
/// Converts `bytecode_buffer` and auxiliary data into a build.
///
pub fn build(
    bytecode_buffer: inkwell::memory_buffer::MemoryBuffer,
    metadata_hash: Option<era_compiler_common::Hash>,
    cbor_data: Option<(String, Vec<(String, semver::Version)>)>,
    assembly_text: Option<String>,
) -> anyhow::Result<Build> {
    let metadata = match (metadata_hash, cbor_data) {
        (Some(era_compiler_common::Hash::IPFS(hash)), Some((cbor_key, cbor_data))) => {
            let cbor = era_compiler_common::CBOR::new(
                Some((
                    era_compiler_common::EraVMMetadataHashType::IPFS,
                    hash.as_bytes(),
                )),
                cbor_key,
                cbor_data,
            );
            cbor.to_vec()
        }
        (None, Some((cbor_key, cbor_data))) => {
            let cbor = era_compiler_common::CBOR::<'_, String>::new(None, cbor_key, cbor_data);
            cbor.to_vec()
        }
        (Some(era_compiler_common::Hash::Keccak256(hash)), _) => hash.to_vec(),
        (_, None) => vec![],
    };

    let bytecode_buffer_with_metadata = if metadata.is_empty() {
        bytecode_buffer
    } else {
        bytecode_buffer
            .append_metadata_eravm(metadata.as_slice())
            .map_err(|error| anyhow::anyhow!("bytecode metadata appending error: {error}"))?
    };
    let bytecode = bytecode_buffer_with_metadata.as_slice().to_vec();

    let build = Build::new(bytecode, metadata, assembly_text);
    Ok(build)
}

///
/// Implemented by items which are translated into LLVM IR.
///
pub trait WriteLLVM {
    ///
    /// Declares the entity in the LLVM IR.
    /// Is usually performed in order to use the item before defining it.
    ///
    fn declare(&mut self, _context: &mut Context) -> anyhow::Result<()> {
        Ok(())
    }

    ///
    /// Translates the entity into LLVM IR.
    ///
    fn into_llvm(self, context: &mut Context) -> anyhow::Result<()>;
}

///
/// The dummy LLVM writable entity.
///
#[derive(Debug, Default, Clone)]
pub struct DummyLLVMWritable {}

impl WriteLLVM for DummyLLVMWritable {
    fn into_llvm(self, _context: &mut Context) -> anyhow::Result<()> {
        Ok(())
    }
}
