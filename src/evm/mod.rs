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
/// Links `bytecode_buffer` with `linker_symbols` and `factory_dependencies`.
///
pub fn link(
    (deploy_bytecode_identifier, deploy_bytecode_buffer): (
        &str,
        inkwell::memory_buffer::MemoryBuffer,
    ),
    (runtime_bytecode_identifier, runtime_bytecode_buffer): (
        &str,
        inkwell::memory_buffer::MemoryBuffer,
    ),
    linker_symbols: &BTreeMap<String, [u8; era_compiler_common::BYTE_LENGTH_ETH_ADDRESS]>,
) -> anyhow::Result<(
    inkwell::memory_buffer::MemoryBuffer,
    inkwell::memory_buffer::MemoryBuffer,
    era_compiler_common::ObjectFormat,
)> {
    if !deploy_bytecode_buffer.is_elf_eravm() && !runtime_bytecode_buffer.is_elf_eravm() {
        return Ok((
            deploy_bytecode_buffer,
            runtime_bytecode_buffer,
            era_compiler_common::ObjectFormat::Raw,
        ));
    }

    let (deploy_bytecode_buffer_linked, runtime_bytecode_buffer_linked) =
        inkwell::memory_buffer::MemoryBuffer::link_module_evm(
            &[&deploy_bytecode_buffer, &runtime_bytecode_buffer],
            &[deploy_bytecode_identifier, runtime_bytecode_identifier],
            linker_symbols,
        )
        .map_err(|error| anyhow::anyhow!("linking: {error}"))?;

    let object_format = if deploy_bytecode_buffer_linked.is_elf_eravm()
        || runtime_bytecode_buffer_linked.is_elf_eravm()
    {
        era_compiler_common::ObjectFormat::ELF
    } else {
        era_compiler_common::ObjectFormat::Raw
    };
    Ok((
        deploy_bytecode_buffer_linked,
        runtime_bytecode_buffer_linked,
        object_format,
    ))
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
