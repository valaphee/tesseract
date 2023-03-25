use std::io::Write;

use crate::{
    types::{BitStorage, VarI32},
    Encode,
};

pub enum PalettedContainer<
    const STORAGE_SIZE: u32,
    const LINEAR_MIN_BITS: u32,
    const LINEAR_MAX_BITS: u32,
    const GLOBAL_BITS: u32,
> {
    SingleValue(u32),
    Linear {
        palette: Vec<u32>,
        storage: BitStorage,
    },
    Global(BitStorage),
}

impl<
        const STORAGE_SIZE: u32,
        const LINEAR_MIN_BITS: u32,
        const LINEAR_MAX_BITS: u32,
        const GLOBAL_BITS: u32,
    > PalettedContainer<STORAGE_SIZE, LINEAR_MIN_BITS, LINEAR_MAX_BITS, GLOBAL_BITS>
{
    pub fn get_and_set(&mut self, index: u32, value: u32) -> u32 {
        match self {
            PalettedContainer::SingleValue(old_value) => {
                let old_value = *old_value;

                // Resize
                if old_value != value {
                    let mut storage = BitStorage::new(STORAGE_SIZE, LINEAR_MIN_BITS);
                    storage.set(index, 1);
                    *self = Self::Linear {
                        palette: vec![old_value, value],
                        storage,
                    }
                }

                old_value
            }
            PalettedContainer::Linear { palette, storage } => {
                let palette_index = if let Some(palette_index) = palette
                    .iter()
                    .position(|&palette_value| palette_value == value)
                {
                    palette_index
                } else {
                    // Resize
                    if palette.len() as u32 >= storage.mask() {
                        *self = if storage.bits() < LINEAR_MAX_BITS {
                            let mut new_storage =
                                BitStorage::new(storage.size(), storage.bits() + 1);
                            for i in 0..new_storage.size() {
                                new_storage.set(i, storage.get(i));
                            }
                            Self::Linear {
                                palette: palette.clone(),
                                storage: new_storage,
                            }
                        } else {
                            let mut new_storage = BitStorage::new(storage.size(), GLOBAL_BITS);
                            for i in 0..new_storage.size() {
                                new_storage.set(i, palette[storage.get(i) as usize]);
                            }
                            Self::Global(new_storage)
                        };
                        return self.get_and_set(index, value);
                    }

                    palette.push(value);
                    palette.len() - 1
                } as u32;
                palette[storage.get_and_set(index, palette_index) as usize]
            }
            PalettedContainer::Global(storage) => storage.get_and_set(index, value),
        }
    }

    pub fn get(&self, index: u32) -> u32 {
        match self {
            PalettedContainer::SingleValue(value) => *value,
            PalettedContainer::Linear { palette, storage } => palette[storage.get(index) as usize],
            PalettedContainer::Global(storage) => storage.get(index),
        }
    }

    pub fn fix(self) -> Self {
        match self {
            PalettedContainer::Linear { palette, storage } => {
                if storage.bits() < LINEAR_MIN_BITS {
                    let mut new_storage = BitStorage::new(storage.size(), LINEAR_MIN_BITS);
                    for i in 0..new_storage.size() {
                        new_storage.set(i, storage.get(i));
                    }
                    Self::Linear {
                        palette,
                        storage: new_storage,
                    }
                } else {
                    Self::Linear { palette, storage }
                }
            }
            _ => self,
        }
    }
}

impl<
        const STORAGE_SIZE: u32,
        const LINEAR_MIN_BITS: u32,
        const LINEAR_MAX_BITS: u32,
        const GLOBAL_BITS: u32,
    > Encode for PalettedContainer<STORAGE_SIZE, LINEAR_MIN_BITS, LINEAR_MAX_BITS, GLOBAL_BITS>
{
    fn encode(&self, output: &mut impl Write) -> crate::Result<()> {
        match self {
            PalettedContainer::SingleValue(value) => {
                0u8.encode(output)?;
                VarI32(*value as i32).encode(output)?;
                VarI32(0).encode(output)?;
            }
            PalettedContainer::Linear { palette, storage } => {
                (storage.bits() as u8).encode(output)?;
                VarI32(palette.len() as i32).encode(output)?;
                for &element in palette {
                    VarI32(element as i32).encode(output)?;
                }
                let data = storage.data();
                VarI32(data.len() as i32).encode(output)?;
                for &element in data {
                    element.encode(output)?;
                }
            }
            PalettedContainer::Global(storage) => {
                (storage.bits() as u8).encode(output)?;
                let data = storage.data();
                VarI32(data.len() as i32).encode(output)?;
                for &element in data {
                    element.encode(output)?;
                }
            }
        }
        Ok(())
    }
}
