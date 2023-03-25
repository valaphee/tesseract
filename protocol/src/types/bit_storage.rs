#[rustfmt::skip]
const MAGIC: [u32; 192] = [
    0b11111111111111111111111111111111, 0b11111111111111111111111111111111, 0,
    0b10000000000000000000000000000000, 0b00000000000000000000000000000000, 0,
    0b01010101010101010101010101010101, 0b01010101010101010101010101010101, 0,
    0b10000000000000000000000000000000, 0b00000000000000000000000000000000, 1,
    0b00110011001100110011001100110011, 0b00110011001100110011001100110011, 0,
    0b00101010101010101010101010101010, 0b00101010101010101010101010101010, 0,
    0b00100100100100100100100100100100, 0b00100100100100100100100100100100, 0,
    0b10000000000000000000000000000000, 0b00000000000000000000000000000000, 2,
    0b00011100011100011100011100011100, 0b00011100011100011100011100011100, 0,
    0b00011001100110011001100110011001, 0b00011001100110011001100110011001, 0,
    0b00010111010001011101000101110100, 0b00010111010001011101000101110100, 0,
    0b00010101010101010101010101010101, 0b00010101010101010101010101010101, 0,
    0b00010011101100010011101100010011, 0b00010011101100010011101100010011, 0,
    0b00010010010010010010010010010010, 0b00010010010010010010010010010010, 0,
    0b00010001000100010001000100010001, 0b00010001000100010001000100010001, 0,
    0b10000000000000000000000000000000, 0b00000000000000000000000000000000, 3,
    0b00001111000011110000111100001111, 0b00001111000011110000111100001111, 0,
    0b00001110001110001110001110001110, 0b00001110001110001110001110001110, 0,
    0b00001101011110010100001101011110, 0b00001101011110010100001101011110, 0,
    0b00001100110011001100110011001100, 0b00001100110011001100110011001100, 0,
    0b00001100001100001100001100001100, 0b00001100001100001100001100001100, 0,
    0b00001011101000101110100010111010, 0b00001011101000101110100010111010, 0,
    0b00001011001000010110010000101100, 0b00001011001000010110010000101100, 0,
    0b00001010101010101010101010101010, 0b00001010101010101010101010101010, 0,
    0b00001010001111010111000010100011, 0b00001010001111010111000010100011, 0,
    0b00001001110110001001110110001001, 0b00001001110110001001110110001001, 0,
    0b00001001011110110100001001011110, 0b00001001011110110100001001011110, 0,
    0b00001001001001001001001001001001, 0b00001001001001001001001001001001, 0,
    0b00001000110100111101110010110000, 0b00001000110100111101110010110000, 0,
    0b00001000100010001000100010001000, 0b00001000100010001000100010001000, 0,
    0b00001000010000100001000010000100, 0b00001000010000100001000010000100, 0,
    0b10000000000000000000000000000000, 0b00000000000000000000000000000000, 4,
    0b00000111110000011111000001111100, 0b00000111110000011111000001111100, 0,
    0b00000111100001111000011110000111, 0b00000111100001111000011110000111, 0,
    0b00000111010100000111010100000111, 0b00000111010100000111010100000111, 0,
    0b00000111000111000111000111000111, 0b00000111000111000111000111000111, 0,
    0b00000110111010110011111001000101, 0b00000110111010110011111001000101, 0,
    0b00000110101111001010000110101111, 0b00000110101111001010000110101111, 0,
    0b00000110100100000110100100000110, 0b00000110100100000110100100000110, 0,
    0b00000110011001100110011001100110, 0b00000110011001100110011001100110, 0,
    0b00000110001111100111000001100011, 0b00000110001111100111000001100011, 0,
    0b00000110000110000110000110000110, 0b00000110000110000110000110000110, 0,
    0b00000101111101000001011111010000, 0b00000101111101000001011111010000, 0,
    0b00000101110100010111010001011101, 0b00000101110100010111010001011101, 0,
    0b00000101101100000101101100000101, 0b00000101101100000101101100000101, 0,
    0b00000101100100001011001000010110, 0b00000101100100001011001000010110, 0,
    0b00000101011100100110001000001010, 0b00000101011100100110001000001010, 0,
    0b00000101010101010101010101010101, 0b00000101010101010101010101010101, 0,
    0b00000101001110010111100000101001, 0b00000101001110010111100000101001, 0,
    0b00000101000111101011100001010001, 0b00000101000111101011100001010001, 0,
    0b00000101000001010000010100000101, 0b00000101000001010000010100000101, 0,
    0b00000100111011000100111011000100, 0b00000100111011000100111011000100, 0,
    0b00000100110101001000011100111110, 0b00000100110101001000011100111110, 0,
    0b00000100101111011010000100101111, 0b00000100101111011010000100101111, 0,
    0b00000100101001111001000001001010, 0b00000100101001111001000001001010, 0,
    0b00000100100100100100100100100100, 0b00000100100100100100100100100100, 0,
    0b00000100011111011100000100011111, 0b00000100011111011100000100011111, 0,
    0b00000100011010011110111001011000, 0b00000100011010011110111001011000, 0,
    0b00000100010101101100011110010111, 0b00000100010101101100011110010111, 0,
    0b00000100010001000100010001000100, 0b00000100010001000100010001000100, 0,
    0b00000100001100100101110001010011, 0b00000100001100100101110001010011, 0,
    0b00000100001000010000100001000010, 0b00000100001000010000100001000010, 0,
    0b00000100000100000100000100000100, 0b00000100000100000100000100000100, 0,
    0b10000000000000000000000000000000, 0b00000000000000000000000000000000, 5
];

pub struct BitStorage {
    size: u32,
    bits: u32,
    data: Vec<u64>,

    mask: u32,
    values_per_long: u32,
    divide_mul: u32,
    divide_add: u32,
    divide_shift: u32,
}

impl BitStorage {
    pub fn new(size: u32, bits: u32) -> Self {
        assert!((1..32).contains(&bits));
        let values_per_long = u64::BITS / bits;
        let magic_index = 3 * (values_per_long - 1) as usize;

        Self {
            size,
            bits,
            data: vec![0; ((size + values_per_long - 1) / values_per_long) as usize],
            mask: (1 << bits) - 1,
            values_per_long,
            divide_mul: MAGIC[magic_index],
            divide_add: MAGIC[magic_index + 1],
            divide_shift: MAGIC[magic_index + 2],
        }
    }

    pub fn from_data(size: u32, data: Vec<u64>) -> Self {
        let values_per_long = if size <= 64 && data.len() == 1 {
            64
        } else {
            (size - 1) / (data.len() as u32 - 1)
        };
        let bits = u64::BITS / values_per_long;
        let magic_index = 3 * (values_per_long - 1) as usize;

        Self {
            size,
            bits,
            data,
            mask: (1 << bits) - 1,
            values_per_long,
            divide_mul: MAGIC[magic_index],
            divide_add: MAGIC[magic_index + 1],
            divide_shift: MAGIC[magic_index + 2],
        }
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn bits(&self) -> u32 {
        self.bits
    }

    pub fn data(&self) -> &[u64] {
        &self.data
    }

    pub fn mask(&self) -> u32 {
        self.mask
    }

    pub fn get_and_set(&mut self, index: u32, value: u32) -> u32 {
        assert!(index < self.size);
        assert!(value < self.mask);
        let cell_index = self.cell_index(index);
        let bit_index = self.bit_index(index, cell_index);
        let cell = &mut self.data[cell_index as usize];
        let old_value = (*cell >> bit_index) as u32 & self.mask;
        *cell = *cell & !(self.mask as u64 >> bit_index) | (value as u64) << bit_index;
        old_value
    }

    pub fn set(&mut self, index: u32, value: u32) {
        assert!(index < self.size);
        assert!(value <= self.mask);
        let cell_index = self.cell_index(index);
        let bit_index = self.bit_index(index, cell_index);
        let cell = &mut self.data[cell_index as usize];
        *cell = *cell & !(self.mask as u64 >> bit_index) | (value as u64) << bit_index;
    }

    pub fn get(&self, index: u32) -> u32 {
        assert!(index < self.size);
        let cell_index = self.cell_index(index);
        let bit_index = self.bit_index(index, cell_index);
        (self.data[cell_index as usize] >> bit_index) as u32 & self.mask
    }

    fn cell_index(&self, index: u32) -> u32 {
        ((index as u64 * self.divide_mul as u64 + self.divide_add as u64) >> 32) as u32
            >> self.divide_shift
    }

    fn bit_index(&self, index: u32, cell_index: u32) -> u32 {
        (index - cell_index * self.values_per_long) * self.bits
    }
}

#[cfg(test)]
mod tests {
    use rand::{rngs::StdRng, Rng, SeedableRng};

    use crate::types::BitStorage;

    #[test]
    fn set_and_get() {
        for bits in 1..32 {
            let mut bit_storage = BitStorage::new(256, bits);
            let mut rng = StdRng::seed_from_u64(0);
            for i in 0..bit_storage.size {
                bit_storage.set(i, rng.gen_range(0..1 << bits));
            }
            rng = StdRng::seed_from_u64(0);
            for i in 0..bit_storage.size {
                assert_eq!(rng.gen_range(0..1 << bits), bit_storage.get(i))
            }
        }
    }
}
