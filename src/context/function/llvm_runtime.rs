//!
//! The LLVM runtime functions.
//!

use inkwell::types::BasicType;

use crate::context::address_space::AddressSpace;
use crate::context::function::declaration::Declaration as FunctionDeclaration;
use crate::context::function::Function;
use crate::context::optimizer::Optimizer;

///
/// The runtime functions, implemented on the LLVM side.
///
/// The functions are automatically linked to the LLVM implementations if the signatures match.
///
#[derive(Debug)]
pub struct LLVMRuntime<'ctx> {
    /// The LLVM personality function, used for exception handling.
    pub personality: FunctionDeclaration<'ctx>,
    /// The LLVM exception throwing function.
    pub cxa_throw: FunctionDeclaration<'ctx>,

    /// The corresponding LLVM runtime function.
    pub add_mod: FunctionDeclaration<'ctx>,
    /// The corresponding LLVM runtime function.
    pub mul_mod: FunctionDeclaration<'ctx>,
    /// The corresponding LLVM runtime function.
    pub sign_extend: FunctionDeclaration<'ctx>,

    /// The corresponding LLVM runtime function.
    pub far_call: FunctionDeclaration<'ctx>,
    /// The corresponding LLVM runtime function.
    pub far_call_byref: FunctionDeclaration<'ctx>,

    /// The corresponding LLVM runtime function.
    pub static_call: FunctionDeclaration<'ctx>,
    /// The corresponding LLVM runtime function.
    pub static_call_byref: FunctionDeclaration<'ctx>,

    /// The corresponding LLVM runtime function.
    pub delegate_call: FunctionDeclaration<'ctx>,
    /// The corresponding LLVM runtime function.
    pub delegate_call_byref: FunctionDeclaration<'ctx>,

    /// The corresponding LLVM runtime function.
    pub mimic_call: FunctionDeclaration<'ctx>,
    /// The corresponding LLVM runtime function.
    pub mimic_call_byref: FunctionDeclaration<'ctx>,
}

impl<'ctx> LLVMRuntime<'ctx> {
    /// The LLVM personality function name.
    pub const FUNCTION_PERSONALITY: &'static str = "__personality";

    /// The LLVM exception throwing function name.
    pub const FUNCTION_CXA_THROW: &'static str = "__cxa_throw";

    /// The corresponding runtime function name.
    pub const FUNCTION_ADDMOD: &'static str = "__addmod";

    /// The corresponding runtime function name.
    pub const FUNCTION_MULMOD: &'static str = "__mulmod";

    /// The corresponding runtime function name.
    pub const FUNCTION_SIGNEXTEND: &'static str = "__signextend";

    /// The corresponding runtime function name.
    pub const FUNCTION_FARCALL: &'static str = "__farcall";

    /// The corresponding runtime function name.
    pub const FUNCTION_STATICCALL: &'static str = "__staticcall";

    /// The corresponding runtime function name.
    pub const FUNCTION_DELEGATECALL: &'static str = "__delegatecall";

    /// The corresponding runtime function name.
    pub const FUNCTION_MIMICCALL: &'static str = "__mimiccall";

    /// The corresponding runtime function name.
    pub const FUNCTION_FARCALL_BYREF: &'static str = "__farcall_byref";

    /// The corresponding runtime function name.
    pub const FUNCTION_STATICCALL_BYREF: &'static str = "__staticcall_byref";

    /// The corresponding runtime function name.
    pub const FUNCTION_DELEGATECALL_BYREF: &'static str = "__delegatecall_byref";

    /// The corresponding runtime function name.
    pub const FUNCTION_MIMICCALL_BYREF: &'static str = "__mimiccall_byref";

    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        llvm: &'ctx inkwell::context::Context,
        module: &inkwell::module::Module<'ctx>,
        optimizer: &Optimizer,
    ) -> Self {
        let personality = Self::declare(
            module,
            Self::FUNCTION_PERSONALITY,
            llvm.i32_type().fn_type(&[], false),
            None,
        );

        let cxa_throw = Self::declare(
            module,
            Self::FUNCTION_CXA_THROW,
            llvm.void_type().fn_type(
                vec![
                    llvm.i8_type()
                        .ptr_type(AddressSpace::Stack.into())
                        .as_basic_type_enum()
                        .into();
                    3
                ]
                .as_slice(),
                false,
            ),
            Some(inkwell::module::Linkage::External),
        );
        Function::set_cxa_throw_attributes(llvm, cxa_throw);

        let add_mod = Self::declare(
            module,
            Self::FUNCTION_ADDMOD,
            llvm.custom_width_int_type(compiler_common::BIT_LENGTH_FIELD as u32)
                .fn_type(
                    vec![
                        llvm.custom_width_int_type(compiler_common::BIT_LENGTH_FIELD as u32)
                            .as_basic_type_enum()
                            .into();
                        3
                    ]
                    .as_slice(),
                    false,
                ),
            Some(inkwell::module::Linkage::External),
        );
        Function::set_default_attributes(llvm, add_mod, optimizer);
        Function::set_llvm_runtime_attributes(llvm, add_mod);

        let mul_mod = Self::declare(
            module,
            Self::FUNCTION_MULMOD,
            llvm.custom_width_int_type(compiler_common::BIT_LENGTH_FIELD as u32)
                .fn_type(
                    vec![
                        llvm.custom_width_int_type(compiler_common::BIT_LENGTH_FIELD as u32)
                            .as_basic_type_enum()
                            .into();
                        3
                    ]
                    .as_slice(),
                    false,
                ),
            Some(inkwell::module::Linkage::External),
        );
        Function::set_default_attributes(llvm, mul_mod, optimizer);
        Function::set_llvm_runtime_attributes(llvm, mul_mod);

        let sign_extend = Self::declare(
            module,
            Self::FUNCTION_SIGNEXTEND,
            llvm.custom_width_int_type(compiler_common::BIT_LENGTH_FIELD as u32)
                .fn_type(
                    vec![
                        llvm.custom_width_int_type(compiler_common::BIT_LENGTH_FIELD as u32)
                            .as_basic_type_enum()
                            .into();
                        2
                    ]
                    .as_slice(),
                    false,
                ),
            Some(inkwell::module::Linkage::External),
        );
        Function::set_default_attributes(llvm, sign_extend, optimizer);
        Function::set_llvm_runtime_attributes(llvm, sign_extend);

        let external_call_arguments: Vec<inkwell::types::BasicMetadataTypeEnum> = vec![
                llvm.custom_width_int_type(compiler_common::BIT_LENGTH_FIELD as u32)
                    .as_basic_type_enum()
                    .into();
                crate::context::function::runtime::entry::Entry::MANDATORY_ARGUMENTS_COUNT
                    + crate::EXTRA_ABI_DATA_SIZE
            ];
        let mut mimic_call_arguments = external_call_arguments.clone();
        mimic_call_arguments.push(
            llvm.custom_width_int_type(compiler_common::BIT_LENGTH_FIELD as u32)
                .as_basic_type_enum()
                .into(),
        );

        let mut external_call_arguments_by_ref: Vec<inkwell::types::BasicMetadataTypeEnum> = vec![
            llvm.custom_width_int_type(compiler_common::BIT_LENGTH_BYTE as u32)
                .ptr_type(AddressSpace::Generic.into())
                .as_basic_type_enum()
                .into(),
            llvm.custom_width_int_type(compiler_common::BIT_LENGTH_FIELD as u32)
                .as_basic_type_enum()
                .into(),
        ];
        external_call_arguments_by_ref.extend::<Vec<inkwell::types::BasicMetadataTypeEnum>>(vec![
            llvm.custom_width_int_type(
                compiler_common::BIT_LENGTH_FIELD as u32
            )
            .as_basic_type_enum()
            .into();
            crate::EXTRA_ABI_DATA_SIZE
        ]);
        let mut mimic_call_arguments_by_ref = external_call_arguments_by_ref.clone();
        mimic_call_arguments_by_ref.push(
            llvm.custom_width_int_type(compiler_common::BIT_LENGTH_FIELD as u32)
                .as_basic_type_enum()
                .into(),
        );

        let external_call_result_type = llvm
            .struct_type(
                &[
                    llvm.custom_width_int_type(compiler_common::BIT_LENGTH_BYTE as u32)
                        .ptr_type(AddressSpace::Generic.into())
                        .as_basic_type_enum(),
                    llvm.bool_type().as_basic_type_enum(),
                ],
                false,
            )
            .as_basic_type_enum();

        let far_call = Self::declare(
            module,
            Self::FUNCTION_FARCALL,
            external_call_result_type.fn_type(external_call_arguments.as_slice(), false),
            Some(inkwell::module::Linkage::External),
        );
        Function::set_default_attributes(llvm, far_call, optimizer);
        let static_call = Self::declare(
            module,
            Self::FUNCTION_STATICCALL,
            external_call_result_type.fn_type(external_call_arguments.as_slice(), false),
            Some(inkwell::module::Linkage::External),
        );
        Function::set_default_attributes(llvm, static_call, optimizer);
        let delegate_call = Self::declare(
            module,
            Self::FUNCTION_DELEGATECALL,
            external_call_result_type.fn_type(external_call_arguments.as_slice(), false),
            Some(inkwell::module::Linkage::External),
        );
        Function::set_default_attributes(llvm, delegate_call, optimizer);
        let mimic_call = Self::declare(
            module,
            Self::FUNCTION_MIMICCALL,
            external_call_result_type.fn_type(mimic_call_arguments.as_slice(), false),
            Some(inkwell::module::Linkage::External),
        );
        Function::set_default_attributes(llvm, mimic_call, optimizer);

        let far_call_byref = Self::declare(
            module,
            Self::FUNCTION_FARCALL_BYREF,
            external_call_result_type.fn_type(external_call_arguments_by_ref.as_slice(), false),
            Some(inkwell::module::Linkage::External),
        );
        Function::set_default_attributes(llvm, far_call_byref, optimizer);
        let static_call_byref = Self::declare(
            module,
            Self::FUNCTION_STATICCALL_BYREF,
            external_call_result_type.fn_type(external_call_arguments_by_ref.as_slice(), false),
            Some(inkwell::module::Linkage::External),
        );
        Function::set_default_attributes(llvm, static_call_byref, optimizer);
        let delegate_call_byref = Self::declare(
            module,
            Self::FUNCTION_DELEGATECALL_BYREF,
            external_call_result_type.fn_type(external_call_arguments_by_ref.as_slice(), false),
            Some(inkwell::module::Linkage::External),
        );
        Function::set_default_attributes(llvm, delegate_call_byref, optimizer);
        let mimic_call_byref = Self::declare(
            module,
            Self::FUNCTION_MIMICCALL_BYREF,
            external_call_result_type.fn_type(mimic_call_arguments_by_ref.as_slice(), false),
            Some(inkwell::module::Linkage::External),
        );
        Function::set_default_attributes(llvm, mimic_call_byref, optimizer);

        Self {
            personality,
            cxa_throw,

            add_mod,
            mul_mod,

            sign_extend,

            far_call,
            static_call,
            delegate_call,
            mimic_call,

            far_call_byref,
            static_call_byref,
            delegate_call_byref,
            mimic_call_byref,
        }
    }

    ///
    /// Declares an LLVM runtime function in the `module`,
    ///
    pub fn declare(
        module: &inkwell::module::Module<'ctx>,
        name: &str,
        r#type: inkwell::types::FunctionType<'ctx>,
        linkage: Option<inkwell::module::Linkage>,
    ) -> FunctionDeclaration<'ctx> {
        let value = module.add_function(name, r#type, linkage);
        FunctionDeclaration::new(r#type, value)
    }

    ///
    /// Modifies the external call function with `is_byref` and `is_system` modifiers.
    ///
    pub fn modify(
        &self,
        function: FunctionDeclaration<'ctx>,
        is_byref: bool,
    ) -> anyhow::Result<FunctionDeclaration<'ctx>> {
        let modified = if function == self.far_call {
            match is_byref {
                false => self.far_call,
                true => self.far_call_byref,
            }
        } else if function == self.static_call {
            match is_byref {
                false => self.static_call,
                true => self.static_call_byref,
            }
        } else if function == self.delegate_call {
            match is_byref {
                false => self.delegate_call,
                true => self.delegate_call_byref,
            }
        } else if function == self.mimic_call {
            match is_byref {
                false => self.mimic_call,
                true => self.mimic_call_byref,
            }
        } else {
            anyhow::bail!(
                "Cannot modify an external call function `{}`",
                function.value.get_name().to_string_lossy()
            );
        };

        Ok(modified)
    }
}
