use crate::word::Word;

const ROM_SIZE: usize = 0x4000;

pub struct Rom {
    rom: [u8; ROM_SIZE],
}

impl Default for Rom {
    fn default() -> Self {
        Rom { rom: [0; ROM_SIZE] }
    }
}

impl Rom {
    pub fn load(&mut self, data: &[u8]) {
        if data.len() != ROM_SIZE {
            panic!();
        }

        self.rom.copy_from_slice(data);
    }

    pub fn read(&mut self, address: Word) -> u8 {
        self.rom[Into::<usize>::into(address)]
    }
}
