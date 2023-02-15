//!
//! The LLVM global value.
//!

use inkwell::types::BasicType;

use crate::AddressSpace;

///
/// The LLVM global value.
///
#[derive(Debug, Clone, Copy)]
pub struct Global<'ctx> {
    /// The global type.
    pub r#type: inkwell::types::BasicTypeEnum<'ctx>,
    /// The global value.
    pub value: inkwell::values::GlobalValue<'ctx>,
}

impl<'ctx> Global<'ctx> {
    ///
    /// A shortcut constructor.
    ///
    pub fn new<T>(module: &inkwell::module::Module<'ctx>, r#type: T, name: &str) -> Self
    where
        T: BasicType<'ctx>,
    {
        let r#type = r#type.as_basic_type_enum();

        let value = module.add_global(r#type, Some(AddressSpace::Stack.into()), name);
        value.set_linkage(inkwell::module::Linkage::Private);
        value.set_visibility(inkwell::GlobalVisibility::Default);
        value.set_externally_initialized(false);

        value.set_initializer(&r#type.const_zero());
        Self { r#type, value }
    }
}
