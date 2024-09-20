//!
//! The LLVM IR generator Yul data.
//!

use std::collections::BTreeMap;

use num::Zero;

///
/// The LLVM IR generator Yul data.
///
/// Describes some data that is only relevant to Yul.
///
#[derive(Debug, Default)]
pub struct YulData {
    /// The EraVM extensions flag.
    /// The call simulations only work if this mode is enabled.
    are_eravm_extensions_enabled: bool,
    /// The list of constant arrays in the code section.
    /// It is a temporary storage used until the finalization method is called.
    const_arrays: BTreeMap<u8, Vec<num::BigUint>>,
}

impl YulData {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(are_eravm_extensions_enabled: bool) -> Self {
        Self {
            are_eravm_extensions_enabled,
            const_arrays: BTreeMap::new(),
        }
    }

    ///
    /// Whether the EraVM extensions is enabled.
    ///
    pub fn are_eravm_extensions_enabled(&self) -> bool {
        self.are_eravm_extensions_enabled
    }

    ///
    /// Declares a temporary constant array representation.
    ///
    pub fn const_array_declare(&mut self, index: u8, size: u16) -> anyhow::Result<()> {
        if self.const_arrays.contains_key(&index) {
            anyhow::bail!("constant array with index {index} is already declared",);
        }

        self.const_arrays
            .insert(index, vec![num::BigUint::zero(); size as usize]);

        Ok(())
    }

    ///
    /// Sets a value in the constant array representation.
    ///
    pub fn const_array_set(
        &mut self,
        index: u8,
        offset: u16,
        value: num::BigUint,
    ) -> anyhow::Result<()> {
        let array = self.const_arrays.get_mut(&index).ok_or_else(|| {
            anyhow::anyhow!("The constant array with index {} is not declared", index)
        })?;
        if offset >= array.len() as u16 {
            anyhow::bail!(
                "constant array with index {index} has size {}, but the offset is {offset}",
                array.len(),
            );
        }
        array[offset as usize] = value;

        Ok(())
    }

    ///
    /// Finalizes the constant array declaration.
    ///
    pub fn const_array_take(&mut self, index: u8) -> anyhow::Result<Vec<num::BigUint>> {
        self.const_arrays.remove(&index).ok_or_else(|| {
            anyhow::anyhow!("The constant array with index {} is not declared", index)
        })
    }
}
