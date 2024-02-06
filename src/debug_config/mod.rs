//!
//! The debug configuration.
//!

pub mod ir_type;

use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::context::code_type::CodeType;

use self::ir_type::IRType;

///
/// The debug configuration.
///
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct DebugConfig {
    /// The directory to dump the IRs to.
    pub output_directory: PathBuf,
}

impl DebugConfig {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(output_directory: PathBuf) -> Self {
        Self { output_directory }
    }

    ///
    /// Dumps the Yul IR.
    ///
    pub fn dump_yul(
        &self,
        contract_path: &str,
        code_type: Option<CodeType>,
        code: &str,
    ) -> anyhow::Result<()> {
        let mut file_path = self.output_directory.to_owned();
        let full_file_name = Self::full_file_name(contract_path, code_type, None, IRType::Yul);
        file_path.push(full_file_name);
        std::fs::write(file_path, code)?;

        Ok(())
    }

    ///
    /// Dumps the EVM legacy assembly IR.
    ///
    pub fn dump_evmla(
        &self,
        contract_path: &str,
        code_type: Option<CodeType>,
        code: &str,
    ) -> anyhow::Result<()> {
        let mut file_path = self.output_directory.to_owned();
        let full_file_name = Self::full_file_name(contract_path, code_type, None, IRType::EVMLA);
        file_path.push(full_file_name);
        std::fs::write(file_path, code)?;

        Ok(())
    }

    ///
    /// Dumps the Ethereal IR.
    ///
    pub fn dump_ethir(
        &self,
        contract_path: &str,
        code_type: Option<CodeType>,
        code: &str,
    ) -> anyhow::Result<()> {
        let mut file_path = self.output_directory.to_owned();
        let full_file_name = Self::full_file_name(contract_path, code_type, None, IRType::EthIR);
        file_path.push(full_file_name);
        std::fs::write(file_path, code)?;

        Ok(())
    }

    ///
    /// Dumps the LLL IR.
    ///
    pub fn dump_lll(
        &self,
        contract_path: &str,
        code_type: Option<CodeType>,
        code: &str,
    ) -> anyhow::Result<()> {
        let mut file_path = self.output_directory.to_owned();
        let full_file_name = Self::full_file_name(contract_path, code_type, None, IRType::LLL);
        file_path.push(full_file_name);
        std::fs::write(file_path, code)?;

        Ok(())
    }

    ///
    /// Dumps the unoptimized LLVM IR.
    ///
    pub fn dump_llvm_ir_unoptimized(
        &self,
        contract_path: &str,
        code_type: Option<CodeType>,
        module: &inkwell::module::Module,
    ) -> anyhow::Result<()> {
        let llvm_code = module.print_to_string().to_string();

        let mut file_path = self.output_directory.to_owned();
        let full_file_name =
            Self::full_file_name(contract_path, code_type, Some("unoptimized"), IRType::LLVM);
        file_path.push(full_file_name);
        std::fs::write(file_path, llvm_code)?;

        Ok(())
    }

    ///
    /// Dumps the optimized LLVM IR.
    ///
    pub fn dump_llvm_ir_optimized(
        &self,
        contract_path: &str,
        code_type: Option<CodeType>,
        module: &inkwell::module::Module,
    ) -> anyhow::Result<()> {
        let llvm_code = module.print_to_string().to_string();

        let mut file_path = self.output_directory.to_owned();
        let full_file_name =
            Self::full_file_name(contract_path, code_type, Some("optimized"), IRType::LLVM);
        file_path.push(full_file_name);
        std::fs::write(file_path, llvm_code)?;

        Ok(())
    }

    ///
    /// Dumps the assembly.
    ///
    pub fn dump_assembly(
        &self,
        contract_path: &str,
        code_type: Option<CodeType>,
        code: &str,
    ) -> anyhow::Result<()> {
        let mut file_path = self.output_directory.to_owned();
        let full_file_name = Self::full_file_name(contract_path, code_type, None, IRType::Assembly);
        file_path.push(full_file_name);
        std::fs::write(file_path, code)?;

        Ok(())
    }

    ///
    /// Creates a full file name, given the contract full path, suffix, and extension.
    ///
    fn full_file_name(
        contract_path: &str,
        code_type: Option<CodeType>,
        suffix: Option<&str>,
        ir_type: IRType,
    ) -> String {
        let mut full_file_name = contract_path.replace('/', "_").replace(':', ".");
        if let Some(code_type) = code_type {
            full_file_name.push('.');
            full_file_name.push_str(code_type.to_string().as_str());
        }
        if let Some(suffix) = suffix {
            full_file_name.push('.');
            full_file_name.push_str(suffix);
        }
        full_file_name.push('.');
        full_file_name.push_str(ir_type.file_extension());
        full_file_name
    }
}
