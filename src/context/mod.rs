//!
//! The LLVM module context trait.
//!

pub mod argument;
pub mod attribute;
pub mod block_key;
pub mod code_type;
pub mod r#loop;

use inkwell::types::BasicType;
use inkwell::values::BasicValue;

use self::code_type::CodeType;
use self::r#loop::Loop;

///
/// The LLVM module context trait.
///
pub trait IContext<'ctx> {
    /// The pointer type.
    type Pointer;

    /// The Solidity extra data type.
    type SolidityData;

    /// The Yul extra data type.
    type YulData;

    /// The EVMLA extra data type.
    type EVMLAData;

    /// The Solidit extra data type.
    type VyperData;

    ///
    /// Returns the inner LLVM context.
    ///
    fn llvm(&self) -> &'ctx inkwell::context::Context;

    ///
    /// Returns the LLVM IR builder.
    ///
    fn builder(&self) -> &inkwell::builder::Builder<'ctx>;

    ///
    /// Returns the current LLVM IR module reference.
    ///
    fn module(&self) -> &inkwell::module::Module<'ctx>;

    ///
    /// Sets the current code type (deploy or runtime).
    ///
    fn set_code_type(&mut self, code_type: CodeType);

    ///
    /// Returns the current code type (deploy or runtime).
    ///
    fn code_type(&self) -> Option<CodeType>;

    ///
    /// Appends a new basic block to the current function.
    ///
    fn append_basic_block(&self, name: &str) -> inkwell::basic_block::BasicBlock<'ctx>;

    ///
    /// Sets the current basic block.
    ///
    fn set_basic_block(&self, block: inkwell::basic_block::BasicBlock<'ctx>);

    ///
    /// Returns the current basic block.
    ///
    fn basic_block(&self) -> inkwell::basic_block::BasicBlock<'ctx>;

    ///
    /// Pushes a new loop context to the stack.
    ///
    fn push_loop(
        &mut self,
        body_block: inkwell::basic_block::BasicBlock<'ctx>,
        continue_block: inkwell::basic_block::BasicBlock<'ctx>,
        join_block: inkwell::basic_block::BasicBlock<'ctx>,
    );

    ///
    /// Pops the current loop context from the stack.
    ///
    fn pop_loop(&mut self);

    ///
    /// Returns the current loop context.
    ///
    fn r#loop(&self) -> &Loop<'ctx>;

    ///
    /// Builds a stack allocation instruction.
    ///
    /// Sets the alignment to 256 bits.
    ///
    fn build_alloca<T>(&self, r#type: T, name: &str) -> Self::Pointer
    where
        T: BasicType<'ctx> + Clone + Copy;

    ///
    /// Builds a stack load instruction.
    ///
    /// Sets the alignment to 256 bits for the stack and 1 bit for the heap, parent, and child.
    ///
    fn build_load(
        &self,
        pointer: Self::Pointer,
        name: &str,
    ) -> inkwell::values::BasicValueEnum<'ctx>;

    ///
    /// Builds a stack store instruction.
    ///
    /// Sets the alignment to 256 bits for the stack and 1 bit for the heap, parent, and child.
    ///
    fn build_store<V>(&self, pointer: Self::Pointer, value: V)
    where
        V: BasicValue<'ctx>;

    ///
    /// Builds a GEP instruction.
    ///
    fn build_gep<T>(
        &self,
        pointer: Self::Pointer,
        indexes: &[inkwell::values::IntValue<'ctx>],
        element_type: T,
        name: &str,
    ) -> Self::Pointer
    where
        T: BasicType<'ctx>;

    ///
    /// Builds a conditional branch.
    ///
    /// Checks if there are no other terminators in the block.
    ///
    fn build_conditional_branch(
        &self,
        comparison: inkwell::values::IntValue<'ctx>,
        then_block: inkwell::basic_block::BasicBlock<'ctx>,
        else_block: inkwell::basic_block::BasicBlock<'ctx>,
    );

    ///
    /// Builds an unconditional branch.
    ///
    /// Checks if there are no other terminators in the block.
    ///
    fn build_unconditional_branch(&self, destination_block: inkwell::basic_block::BasicBlock<'ctx>);

    ///
    /// Builds a return.
    ///
    /// Checks if there are no other terminators in the block.
    ///
    fn build_return(&self, value: Option<&dyn BasicValue<'ctx>>);

    ///
    /// Builds an unreachable.
    ///
    /// Checks if there are no other terminators in the block.
    ///
    fn build_unreachable(&self);

    ///
    /// Returns a boolean type constant.
    ///
    fn bool_const(&self, value: bool) -> inkwell::values::IntValue<'ctx>;

    ///
    /// Returns an integer type constant.
    ///
    fn integer_const(&self, bit_length: usize, value: u64) -> inkwell::values::IntValue<'ctx>;

    ///
    /// Returns a 256-bit field type constant.
    ///
    fn field_const(&self, value: u64) -> inkwell::values::IntValue<'ctx>;

    ///
    /// Returns a 256-bit field type undefined value.
    ///
    fn field_undef(&self) -> inkwell::values::IntValue<'ctx>;

    ///
    /// Returns a field type constant from a decimal string.
    ///
    fn field_const_str_dec(&self, value: &str) -> inkwell::values::IntValue<'ctx>;

    ///
    /// Returns a field type constant from a hexadecimal string.
    ///
    fn field_const_str_hex(&self, value: &str) -> inkwell::values::IntValue<'ctx>;

    ///
    /// Returns the void type.
    ///
    fn void_type(&self) -> inkwell::types::VoidType<'ctx>;

    ///
    /// Returns the boolean type.
    ///
    fn bool_type(&self) -> inkwell::types::IntType<'ctx>;

    ///
    /// Returns the default byte type.
    ///
    fn byte_type(&self) -> inkwell::types::IntType<'ctx>;

    ///
    /// Returns the integer type of the specified bit-length.
    ///
    fn integer_type(&self, bit_length: usize) -> inkwell::types::IntType<'ctx>;

    ///
    /// Returns the default field type.
    ///
    fn field_type(&self) -> inkwell::types::IntType<'ctx>;

    ///
    /// Returns the array type with the specified length.
    ///
    fn array_type<T>(&self, element_type: T, length: usize) -> inkwell::types::ArrayType<'ctx>
    where
        T: BasicType<'ctx>;

    ///
    /// Returns the structure type with specified fields.
    ///
    fn structure_type<T>(&self, field_types: &[T]) -> inkwell::types::StructType<'ctx>
    where
        T: BasicType<'ctx>;

    ///
    /// Sets the Solidity data.
    ///
    fn set_solidity_data(&mut self, data: Self::SolidityData);

    ///
    /// Returns the Solidity data reference.
    ///
    /// # Panics
    /// If the Solidity data has not been initialized.
    ///
    fn solidity(&self) -> &Self::SolidityData;

    ///
    /// Returns the Solidity data mutable reference.
    ///
    /// # Panics
    /// If the Solidity data has not been initialized.
    ///
    fn solidity_mut(&mut self) -> &mut Self::SolidityData;

    ///
    /// Sets the Yul data.
    ///
    fn set_yul_data(&mut self, data: Self::YulData);

    ///
    /// Returns the Yul data reference.
    ///
    /// # Panics
    /// If the Yul data has not been initialized.
    ///
    fn yul(&self) -> &Self::YulData;

    ///
    /// Returns the Yul data mutable reference.
    ///
    /// # Panics
    /// If the Yul data has not been initialized.
    ///
    fn yul_mut(&mut self) -> &mut Self::YulData;

    ///
    /// Sets the EVM legacy assembly data.
    ///
    fn set_evmla_data(&mut self, data: Self::EVMLAData);

    ///
    /// Returns the EVM legacy assembly data reference.
    ///
    /// # Panics
    /// If the EVM data has not been initialized.
    ///
    fn evmla(&self) -> &Self::EVMLAData;

    ///
    /// Returns the EVM legacy assembly data mutable reference.
    ///
    /// # Panics
    /// If the EVM data has not been initialized.
    ///
    fn evmla_mut(&mut self) -> &mut Self::EVMLAData;

    ///
    /// Sets the EVM legacy assembly data.
    ///
    fn set_vyper_data(&mut self, data: Self::VyperData);

    ///
    /// Returns the Vyper data reference.
    ///
    /// # Panics
    /// If the Vyper data has not been initialized.
    ///
    fn vyper(&self) -> &Self::VyperData;
}
