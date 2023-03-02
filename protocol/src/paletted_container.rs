use std::io::Write;

use crate::{bit_storage::BitStorage, types::VarInt, Encode};

pub enum PalettedContainer {
    SingleValue {
        value: u32,

        // Configuration
        storage_size: u32,
        linear_min_bits: u32,
        linear_max_bits: u32,
        global_bits: u32,
    },
    Linear {
        palette: Vec<u32>,
        storage: BitStorage,

        // Configuration
        linear_max_bits: u32,
        global_bits: u32,
    },
    Global(BitStorage),
}

impl PalettedContainer {
    pub fn get_and_set(&mut self, index: u32, value: u32) -> u32 {
        match self {
            PalettedContainer::SingleValue {
                value: old_value,
                storage_size,
                linear_min_bits,
                linear_max_bits,
                global_bits,
            } => {
                let old_value = *old_value;

                // Resize
                if old_value != value {
                    let mut storage = BitStorage::new(*storage_size, *linear_min_bits);
                    storage.set(index, 1);
                    *self = Self::Linear {
                        palette: vec![old_value, value],
                        storage,

                        linear_max_bits: *linear_max_bits,
                        global_bits: *global_bits,
                    }
                }

                old_value
            }
            PalettedContainer::Linear {
                palette,
                storage,
                linear_max_bits,
                global_bits,
            } => {
                let palette_index = if let Some(palette_index) = palette
                    .iter()
                    .position(|&palette_value| palette_value == value)
                {
                    palette_index
                } else {
                    // Resize
                    if palette.len() as u32 >= storage.mask() {
                        *self = if storage.bits() < 8 {
                            let mut new_storage =
                                BitStorage::new(storage.size(), storage.bits() + 1);
                            for i in 0..new_storage.size() {
                                new_storage.set(i, storage.get(i));
                            }
                            Self::Linear {
                                palette: palette.clone(),
                                storage: new_storage,

                                linear_max_bits: *linear_max_bits,
                                global_bits: *global_bits,
                            }
                        } else {
                            let mut new_storage = BitStorage::new(storage.size(), 15);
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
            PalettedContainer::SingleValue { value, .. } => *value,
            PalettedContainer::Linear {
                palette, storage, ..
            } => palette[storage.get(index) as usize],
            PalettedContainer::Global(storage) => storage.get(index),
        }
    }
}

impl Encode for PalettedContainer {
    fn encode<W: Write>(&self, output: &mut W) -> crate::Result<()> {
        match self {
            PalettedContainer::SingleValue { value, .. } => {
                0u8.encode(output)?;
                VarInt(*value as i32).encode(output)?;
                VarInt(0).encode(output)?;
            }
            PalettedContainer::Linear {
                palette, storage, ..
            } => {
                (storage.bits() as u8).encode(output)?;
                VarInt(palette.len() as i32).encode(output)?;
                for &element in palette {
                    VarInt(element as i32).encode(output)?;
                }
                let data = storage.data();
                VarInt(data.len() as i32).encode(output)?;
                for &element in data {
                    element.encode(output)?;
                }
            }
            PalettedContainer::Global(storage) => {
                (storage.bits() as u8).encode(output)?;
                let data = storage.data();
                VarInt(data.len() as i32).encode(output)?;
                for &element in data {
                    element.encode(output)?;
                }
            }
        }
        Ok(())
    }
}
