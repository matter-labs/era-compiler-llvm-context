//!
//! The `exit` function.
//!

use inkwell::types::BasicType;
use inkwell::values::BasicValue;

use crate::context::function::declaration::Declaration as FunctionDeclaration;
use crate::context::function::intrinsics::Intrinsics;
use crate::context::function::Function;
use crate::context::Context;
use crate::Dependency;
use crate::WriteLLVM;

///
/// The `exit` function.
///
#[derive(Debug)]
pub struct Exit {
    /// The name of the inner function used for for the low-level return or revert.
    return_function: String,
    /// The function name with the low-level function name as a suffix.
    name: String,
}

impl Exit {
    /// The offset argument index.
    pub const ARGUMENT_INDEX_OFFSET: usize = 0;

    /// The length argument index.
    pub const ARGUMENT_INDEX_LENGTH: usize = 1;

    /// The auxiliary heap marker argument index.
    pub const ARGUMENT_INDEX_AUXILIARY_HEAP_MARKER: usize = 2;

    ///
    /// A shortcut constructor.
    ///
    pub fn new<'ctx, D>(
        context: &Context<'ctx, D>,
        return_function: FunctionDeclaration<'ctx>,
    ) -> Self
    where
        D: Dependency,
    {
        Self {
            return_function: return_function
                .value
                .get_name()
                .to_string_lossy()
                .to_string(),
            name: Self::name(context, return_function),
        }
    }

    ///
    /// Returns the function name.
    ///
    pub fn name<'ctx, D>(
        context: &Context<'ctx, D>,
        return_function: FunctionDeclaration<'ctx>,
    ) -> String
    where
        D: Dependency,
    {
        let suffix = match return_function {
            return_function if return_function == context.intrinsics().r#return => "return",
            return_function if return_function == context.intrinsics().revert => "revert",
            return_function => panic!(
                "Invalid exit inner function `{}`",
                return_function.value.get_name().to_string_lossy()
            ),
        };
        format!("__exit_{suffix}")
    }

    ///
    /// Returns the low-level call function.
    ///
    fn inner_function<'ctx, D>(&self, context: &Context<'ctx, D>) -> FunctionDeclaration<'ctx>
    where
        D: Dependency,
    {
        match self.return_function.as_str() {
            name if name == Intrinsics::FUNCTION_RETURN => context.intrinsics().r#return,
            name if name == Intrinsics::FUNCTION_REVERT => context.intrinsics().revert,
            name => panic!("Invalid exit inner function `{name}`"),
        }
    }
}

impl<D> WriteLLVM<D> for Exit
where
    D: Dependency,
{
    fn declare(&mut self, context: &mut Context<D>) -> anyhow::Result<()> {
        let function_type = context.function_type(
            vec![
                context.field_type().as_basic_type_enum(),
                context.field_type().as_basic_type_enum(),
                context.field_type().as_basic_type_enum(),
            ],
            0,
            false,
        );
        let function = context.add_function(
            self.name.as_str(),
            function_type,
            0,
            Some(inkwell::module::Linkage::Private),
        )?;
        Function::set_frontend_runtime_attributes(context.llvm, function.borrow().declaration());

        Ok(())
    }

    fn into_llvm(self, context: &mut Context<D>) -> anyhow::Result<()> {
        context.set_current_function(self.name.as_str())?;

        let offset = context
            .current_function()
            .borrow()
            .get_nth_param(Self::ARGUMENT_INDEX_OFFSET)
            .into_int_value();
        let length = context
            .current_function()
            .borrow()
            .get_nth_param(Self::ARGUMENT_INDEX_LENGTH)
            .into_int_value();
        let auxiliary_heap_marker = context
            .current_function()
            .borrow()
            .get_nth_param(Self::ARGUMENT_INDEX_AUXILIARY_HEAP_MARKER)
            .into_int_value();

        context.set_basic_block(context.current_function().borrow().entry_block());
        let offset = crate::utils::clamp(
            context,
            offset,
            context.field_const(u32::MAX as u64),
            "exit_offset",
        )?;
        let length = crate::utils::clamp(
            context,
            length,
            context.field_const(u32::MAX as u64),
            "exit_length",
        )?;

        let offset_shifted = context.builder().build_left_shift(
            offset,
            context.field_const((compiler_common::BIT_LENGTH_X32 * 2) as u64),
            "exit_offset_shifted",
        );
        let length_shifted = context.builder().build_left_shift(
            length,
            context.field_const((compiler_common::BIT_LENGTH_X32 * 3) as u64),
            "exit_length_shifted",
        );

        let abi_data =
            context
                .builder()
                .build_int_add(offset_shifted, length_shifted, "exit_abi_data");
        let abi_data_add_auxiliary_heap_marker = context.builder().build_int_add(
            abi_data,
            auxiliary_heap_marker,
            "exit_abi_data_add_heap_auxiliary_marker",
        );

        context.build_call(
            self.inner_function(context),
            &[abi_data_add_auxiliary_heap_marker.as_basic_value_enum()],
            self.return_function.as_str(),
        );
        context.build_unreachable();

        context.set_basic_block(context.current_function().borrow().return_block());
        context.build_unreachable();

        Ok(())
    }
}
