//!
//! The LLVM EVM context library.
//!

pub mod r#const;
pub mod context;
pub mod instructions;
pub mod warning;

use std::collections::BTreeMap;

use self::context::Context;

///
/// Initializes the EVM target machine.
///
pub fn initialize_target() {
    inkwell::targets::Target::initialize_evm(&inkwell::targets::InitializationConfig::default());
}

///
/// Assembles the main buffer and its dependencies from `bytecode_buffers`.
///
pub fn assemble(
    bytecode_buffers: &[&inkwell::memory_buffer::MemoryBuffer],
    bytecode_buffer_ids: &[&str],
    code_segment: era_compiler_common::CodeSegment,
) -> anyhow::Result<inkwell::memory_buffer::MemoryBuffer> {
    let code_segment = match code_segment {
        era_compiler_common::CodeSegment::Deploy => inkwell::memory_buffer::CodeSegment::Deploy,
        era_compiler_common::CodeSegment::Runtime => inkwell::memory_buffer::CodeSegment::Runtime,
    };
    inkwell::memory_buffer::MemoryBuffer::assembly_evm(
        bytecode_buffers,
        bytecode_buffer_ids,
        code_segment,
    )
    .map_err(|error| anyhow::anyhow!("linking: {error}"))
}

///
/// Links `bytecode_buffer` with `linker_symbols.
///
pub fn link(
    bytecode_buffer: inkwell::memory_buffer::MemoryBuffer,
    linker_symbols: &BTreeMap<String, [u8; era_compiler_common::BYTE_LENGTH_ETH_ADDRESS]>,
) -> anyhow::Result<(
    inkwell::memory_buffer::MemoryBuffer,
    era_compiler_common::ObjectFormat,
)> {
    if !bytecode_buffer.is_elf_evm() {
        return Ok((bytecode_buffer, era_compiler_common::ObjectFormat::Raw));
    }

    let bytecode_buffer_linked = bytecode_buffer
        .link_evm(linker_symbols)
        .map_err(|error| anyhow::anyhow!("linking: {error}"))?;

    let object_format = if bytecode_buffer_linked.is_elf_evm() {
        era_compiler_common::ObjectFormat::ELF
    } else {
        era_compiler_common::ObjectFormat::Raw
    };
    Ok((bytecode_buffer_linked, object_format))
}

///
/// Returns a minimal EVM deploy code returning the specified runtime code length.
///
pub fn minimal_deploy_code(runtime_code_length: usize) -> Vec<u8> {
    assert!(
        runtime_code_length <= crate::evm_const::DEPLOY_CODE_SIZE_LIMIT,
        "Runtime code length exceeds the limit of {}B",
        crate::evm_const::DEPLOY_CODE_SIZE_LIMIT,
    );

    static MINIMAL_DEPLOY_CODE: &[u8] = &[
        0x61, 0x00, 0x00, // PUSH2 <runtime_length> (placeholder, big-endian)
        0x60, 0x00, // PUSH1 0x00             (dest in memory = 0)
        0x60, 0x0d, // PUSH1 0x0d             (offset where runtime code begins)
        0x82, // DUP3                   (duplicate <runtime_length>)
        0x39, // CODECOPY               (codecopy(0, 0x0d, <runtime_length>))
        0x60, 0x00, // PUSH1 0x00             (return from memory offset = 0)
        0x90, // SWAP1                  (put <runtime_length> on top)
        0xF3, // RETURN                 (return memory[0..length])
    ];

    let mut minimal_deploy_code = MINIMAL_DEPLOY_CODE.to_vec();
    let runtime_length = (runtime_code_length as u16).to_be_bytes();
    minimal_deploy_code[1] = runtime_length[0];
    minimal_deploy_code[2] = runtime_length[1];
    minimal_deploy_code
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
