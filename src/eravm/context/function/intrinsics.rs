//!
//! The LLVM intrinsic functions.
//!

use inkwell::types::BasicType;

use crate::eravm::context::address_space::AddressSpace;
use crate::eravm::context::function::declaration::Declaration as FunctionDeclaration;

///
/// The LLVM intrinsic functions, implemented in the LLVM back-end.
///
/// Most of them are translated directly into bytecode instructions.
///
#[derive(Debug)]
pub struct Intrinsics<'ctx> {
    /// The event emitting.
    pub event: FunctionDeclaration<'ctx>,
    /// The L1 interactor.
    pub to_l1: FunctionDeclaration<'ctx>,
    /// The precompile call.
    pub precompile: FunctionDeclaration<'ctx>,
    /// The near call with ABI data.
    pub near_call: FunctionDeclaration<'ctx>,
    /// The current contract's address.
    pub address: FunctionDeclaration<'ctx>,
    /// The caller's address.
    pub caller: FunctionDeclaration<'ctx>,
    /// The address where the current contract's code is deployed.
    pub code_source: FunctionDeclaration<'ctx>,
    /// The other data: FunctionDeclaration<'ctx>, including the block information and VM state.
    pub meta: FunctionDeclaration<'ctx>,
    /// The remaining amount of gas.
    pub gas_left: FunctionDeclaration<'ctx>,
    /// The abstract `u128` getter.
    pub get_u128: FunctionDeclaration<'ctx>,
    /// The abstract `u128` setter.
    pub set_u128: FunctionDeclaration<'ctx>,
    /// The public data price setter.
    pub set_pubdata_price: FunctionDeclaration<'ctx>,
    /// The transaction counter incrementor.
    pub increment_tx_counter: FunctionDeclaration<'ctx>,
    /// The pointer shrink.
    pub pointer_shrink: FunctionDeclaration<'ctx>,
    /// The pointer pack.
    pub pointer_pack: FunctionDeclaration<'ctx>,

    /// The storage load.
    pub storage_load: FunctionDeclaration<'ctx>,
    /// The storage store.
    pub storage_store: FunctionDeclaration<'ctx>,

    /// The long return.
    pub r#return: FunctionDeclaration<'ctx>,
    /// The long revert.
    pub revert: FunctionDeclaration<'ctx>,

    /// The memory copy within the heap.
    pub memory_copy: FunctionDeclaration<'ctx>,
    /// The memory copy from a generic page.
    pub memory_copy_from_generic: FunctionDeclaration<'ctx>,
}

impl<'ctx> Intrinsics<'ctx> {
    /// The corresponding intrinsic function name.
    pub const FUNCTION_EVENT: &'static str = "llvm.syncvm.event";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_L1: &'static str = "llvm.syncvm.tol1";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_PRECOMPILE: &'static str = "llvm.syncvm.precompile";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_NEAR_CALL: &'static str = "llvm.syncvm.nearcall";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_ADDRESS: &'static str = "llvm.syncvm.this";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_CALLER: &'static str = "llvm.syncvm.caller";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_CODE_SOURCE: &'static str = "llvm.syncvm.codesource";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_META: &'static str = "llvm.syncvm.meta";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_GAS_LEFT: &'static str = "llvm.syncvm.gasleft";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_GET_U128: &'static str = "llvm.syncvm.getu128";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_SET_U128: &'static str = "llvm.syncvm.setu128";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_SET_PUBDATA_PRICE: &'static str = "llvm.syncvm.setpubdataprice";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_INCREMENT_TX_COUNTER: &'static str = "llvm.syncvm.inctx";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_POINTER_SHRINK: &'static str = "llvm.syncvm.ptr.shrink";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_POINTER_PACK: &'static str = "llvm.syncvm.ptr.pack";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_STORAGE_LOAD: &'static str = "llvm.syncvm.sload";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_STORAGE_STORE: &'static str = "llvm.syncvm.sstore";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_RETURN: &'static str = "llvm.syncvm.return";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_REVERT: &'static str = "llvm.syncvm.revert";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_MEMORY_COPY: &'static str = "llvm.memcpy.p1.p1.i256";

    /// The corresponding intrinsic function name.
    pub const FUNCTION_MEMORY_COPY_FROM_GENERIC: &'static str = "llvm.memcpy.p3.p1.i256";

    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        llvm: &'ctx inkwell::context::Context,
        module: &inkwell::module::Module<'ctx>,
    ) -> Self {
        let void_type = llvm.void_type();
        let bool_type = llvm.bool_type();
        let byte_type = llvm.custom_width_int_type(compiler_common::BIT_LENGTH_BYTE as u32);
        let field_type = llvm.custom_width_int_type(compiler_common::BIT_LENGTH_FIELD as u32);
        let stack_field_pointer_type = field_type.ptr_type(AddressSpace::Stack.into());
        let heap_field_pointer_type = byte_type.ptr_type(AddressSpace::Heap.into());
        let generic_byte_pointer_type = byte_type.ptr_type(AddressSpace::Generic.into());

        let event = Self::declare(
            llvm,
            module,
            Self::FUNCTION_EVENT,
            void_type.fn_type(
                &[
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let to_l1 = Self::declare(
            llvm,
            module,
            Self::FUNCTION_L1,
            void_type.fn_type(
                &[
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let precompile = Self::declare(
            llvm,
            module,
            Self::FUNCTION_PRECOMPILE,
            field_type.fn_type(
                &[
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let near_call = Self::declare(
            llvm,
            module,
            Self::FUNCTION_NEAR_CALL,
            field_type.fn_type(
                &[
                    stack_field_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                true,
            ),
        );
        let address = Self::declare(
            llvm,
            module,
            Self::FUNCTION_ADDRESS,
            field_type.fn_type(&[], false),
        );
        let caller = Self::declare(
            llvm,
            module,
            Self::FUNCTION_CALLER,
            field_type.fn_type(&[], false),
        );
        let code_source = Self::declare(
            llvm,
            module,
            Self::FUNCTION_CODE_SOURCE,
            field_type.fn_type(&[], false),
        );
        let meta = Self::declare(
            llvm,
            module,
            Self::FUNCTION_META,
            field_type.fn_type(&[], false),
        );
        let gas_left = Self::declare(
            llvm,
            module,
            Self::FUNCTION_GAS_LEFT,
            field_type.fn_type(&[], false),
        );
        let get_u128 = Self::declare(
            llvm,
            module,
            Self::FUNCTION_GET_U128,
            field_type.fn_type(&[], false),
        );
        let set_u128 = Self::declare(
            llvm,
            module,
            Self::FUNCTION_SET_U128,
            void_type.fn_type(&[field_type.as_basic_type_enum().into()], false),
        );
        let set_pubdata_price = Self::declare(
            llvm,
            module,
            Self::FUNCTION_SET_PUBDATA_PRICE,
            void_type.fn_type(&[field_type.as_basic_type_enum().into()], false),
        );
        let increment_tx_counter = Self::declare(
            llvm,
            module,
            Self::FUNCTION_INCREMENT_TX_COUNTER,
            void_type.fn_type(&[], false),
        );
        let pointer_shrink = Self::declare(
            llvm,
            module,
            Self::FUNCTION_POINTER_SHRINK,
            generic_byte_pointer_type.fn_type(
                &[
                    generic_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let pointer_pack = Self::declare(
            llvm,
            module,
            Self::FUNCTION_POINTER_PACK,
            generic_byte_pointer_type.fn_type(
                &[
                    generic_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );

        let storage_load = Self::declare(
            llvm,
            module,
            Self::FUNCTION_STORAGE_LOAD,
            field_type.fn_type(&[field_type.as_basic_type_enum().into()], false),
        );
        let storage_store = Self::declare(
            llvm,
            module,
            Self::FUNCTION_STORAGE_STORE,
            void_type.fn_type(
                &[
                    field_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );

        let r#return = Self::declare(
            llvm,
            module,
            Self::FUNCTION_RETURN,
            void_type.fn_type(&[field_type.as_basic_type_enum().into()], false),
        );
        let revert = Self::declare(
            llvm,
            module,
            Self::FUNCTION_REVERT,
            void_type.fn_type(&[field_type.as_basic_type_enum().into()], false),
        );

        let memory_copy = Self::declare(
            llvm,
            module,
            Self::FUNCTION_MEMORY_COPY,
            void_type.fn_type(
                &[
                    heap_field_pointer_type.as_basic_type_enum().into(),
                    heap_field_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    bool_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );
        let memory_copy_from_generic = Self::declare(
            llvm,
            module,
            Self::FUNCTION_MEMORY_COPY_FROM_GENERIC,
            void_type.fn_type(
                &[
                    heap_field_pointer_type.as_basic_type_enum().into(),
                    generic_byte_pointer_type.as_basic_type_enum().into(),
                    field_type.as_basic_type_enum().into(),
                    bool_type.as_basic_type_enum().into(),
                ],
                false,
            ),
        );

        Self {
            event,
            to_l1,
            precompile,
            near_call,
            address,
            caller,
            code_source,
            meta,
            gas_left,
            get_u128,
            set_u128,
            set_pubdata_price,
            increment_tx_counter,
            pointer_shrink,
            pointer_pack,

            storage_load,
            storage_store,

            r#return,
            revert,

            memory_copy,
            memory_copy_from_generic,
        }
    }

    ///
    /// Finds the specified LLVM intrinsic function in the target and returns its declaration.
    ///
    pub fn declare(
        llvm: &'ctx inkwell::context::Context,
        module: &inkwell::module::Module<'ctx>,
        name: &str,
        r#type: inkwell::types::FunctionType<'ctx>,
    ) -> FunctionDeclaration<'ctx> {
        let intrinsic = inkwell::intrinsics::Intrinsic::find(name)
            .unwrap_or_else(|| panic!("Intrinsic function `{name}` does not exist"));
        let argument_types = Self::argument_types(llvm, name);
        let value = intrinsic
            .get_declaration(module, argument_types.as_slice())
            .unwrap_or_else(|| panic!("Intrinsic function `{name}` declaration error"));
        FunctionDeclaration::new(r#type, value)
    }

    ///
    /// Returns the LLVM types for selecting via the signature.
    ///
    pub fn argument_types(
        llvm: &'ctx inkwell::context::Context,
        name: &str,
    ) -> Vec<inkwell::types::BasicTypeEnum<'ctx>> {
        let field_type = llvm.custom_width_int_type(compiler_common::BIT_LENGTH_FIELD as u32);

        match name {
            name if name == Self::FUNCTION_MEMORY_COPY => vec![
                field_type
                    .ptr_type(AddressSpace::Heap.into())
                    .as_basic_type_enum(),
                field_type
                    .ptr_type(AddressSpace::Heap.into())
                    .as_basic_type_enum(),
                field_type.as_basic_type_enum(),
            ],
            name if name == Self::FUNCTION_MEMORY_COPY_FROM_GENERIC => vec![
                field_type
                    .ptr_type(AddressSpace::Heap.into())
                    .as_basic_type_enum(),
                field_type
                    .ptr_type(AddressSpace::Generic.into())
                    .as_basic_type_enum(),
                field_type.as_basic_type_enum(),
            ],
            _ => vec![],
        }
    }
}
