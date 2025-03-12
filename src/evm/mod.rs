//!
//! The LLVM EVM context library.
//!

pub mod r#const;
pub mod context;
pub mod instructions;

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
