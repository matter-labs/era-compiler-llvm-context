//!
//! The debug configuration.
//!

pub mod ir_type;

use std::path::PathBuf;

use self::ir_type::IRType;

///
/// The debug configuration.
///
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
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
    /// Create a subdirectory and return a copy of `DebugConfig` pointing there.
    ///
    pub fn create_subdirectory(&self, directory_name: &str) -> anyhow::Result<Self> {
        let sanitized_name = Self::sanitize_filename_fragment(directory_name);
        let subdirectory_path = self.output_directory.join(sanitized_name.as_str());
        std::fs::create_dir_all(subdirectory_path.as_path())?;
        Ok(Self {
            output_directory: subdirectory_path,
        })
    }

    ///
    /// Dumps the Yul IR.
    ///
    pub fn dump_yul(
        &self,
        contract_path: &str,
        code_segment: Option<era_compiler_common::CodeSegment>,
        code: &str,
    ) -> anyhow::Result<()> {
        let mut file_path = self.output_directory.to_owned();
        let full_file_name = Self::full_file_name(contract_path, code_segment, None, IRType::Yul);
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
        code_segment: Option<era_compiler_common::CodeSegment>,
        code: &str,
    ) -> anyhow::Result<()> {
        let mut file_path = self.output_directory.to_owned();
        let full_file_name = Self::full_file_name(contract_path, code_segment, None, IRType::EVMLA);
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
        code_segment: Option<era_compiler_common::CodeSegment>,
        code: &str,
    ) -> anyhow::Result<()> {
        let mut file_path = self.output_directory.to_owned();
        let full_file_name = Self::full_file_name(contract_path, code_segment, None, IRType::EthIR);
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
        code_segment: Option<era_compiler_common::CodeSegment>,
        code: &str,
    ) -> anyhow::Result<()> {
        let mut file_path = self.output_directory.to_owned();
        let full_file_name = Self::full_file_name(contract_path, code_segment, None, IRType::LLL);
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
        code_segment: Option<era_compiler_common::CodeSegment>,
        module: &inkwell::module::Module,
        is_fallback_to_size: bool,
    ) -> anyhow::Result<()> {
        let llvm_code = module.print_to_string().to_string();

        let mut suffix = "unoptimized".to_owned();
        if is_fallback_to_size {
            suffix.push_str("_fallback_to_size");
        }

        let mut file_path = self.output_directory.to_owned();
        let full_file_name = Self::full_file_name(
            contract_path,
            code_segment,
            Some(suffix.as_str()),
            IRType::LLVM,
        );
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
        code_segment: Option<era_compiler_common::CodeSegment>,
        module: &inkwell::module::Module,
        is_fallback_to_size: bool,
    ) -> anyhow::Result<()> {
        let llvm_code = module.print_to_string().to_string();

        let mut suffix = "optimized".to_owned();
        if is_fallback_to_size {
            suffix.push_str("_fallback_to_size");
        }

        let mut file_path = self.output_directory.to_owned();
        let full_file_name = Self::full_file_name(
            contract_path,
            code_segment,
            Some(suffix.as_str()),
            IRType::LLVM,
        );
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
        code_segment: Option<era_compiler_common::CodeSegment>,
        code: &str,
    ) -> anyhow::Result<()> {
        let mut file_path = self.output_directory.to_owned();
        let full_file_name =
            Self::full_file_name(contract_path, code_segment, None, IRType::Assembly);
        file_path.push(full_file_name);
        std::fs::write(file_path, code)?;

        Ok(())
    }

    ///
    /// Rules to encode a string into a valid filename.
    ///
    fn sanitize_filename_fragment(string: &str) -> String {
        string.replace(['/', ' ', '\t'], "_").replace(':', ".")
    }

    ///
    /// Creates a full file name, given the contract full path, suffix, and extension.
    ///
    fn full_file_name(
        contract_path: &str,
        code_segment: Option<era_compiler_common::CodeSegment>,
        suffix: Option<&str>,
        ir_type: IRType,
    ) -> String {
        let mut full_file_name = Self::sanitize_filename_fragment(contract_path);

        if let Some(code_segment) = code_segment {
            full_file_name.push('.');
            full_file_name.push_str(code_segment.to_string().as_str());
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
