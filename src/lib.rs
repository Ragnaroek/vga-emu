pub mod screen;

const PLANE_SIZE: usize = 0xFFFF; //64KiB

pub struct VGA {
    pub sc_reg: [u8; 5],
    pub gc_reg: [u8; 9],
    pub mem: [[u8; PLANE_SIZE]; 4],
}

pub fn new() -> VGA {
    VGA {
        sc_reg: [0; 5],
        gc_reg: [0; 9],
        mem: [[0; PLANE_SIZE]; 4],
    }
}

//Sequence Controller Register
pub enum SCReg {
    Reset = 0,
    ClockingMode = 1,
    MapMask = 2,
    CharacterMapSelect = 3,
    SequencerMemoryMode = 4,
}

//Graphics Controller Register
pub enum GCReg {
    SetReset = 0,
    EnableSetReset = 1,
    ColorCompare = 2,
    DataRotate = 3,
    ReadMapSelect = 4,
    GraphicsMode = 5,
    MiscGraphics = 6,
    ColorDontCare = 7,
    BitMask = 8,
}

impl VGA {
    pub fn set_sc_data(&mut self, reg: SCReg, v: u8) {
        self.sc_reg[reg as usize] = v;
    }
    pub fn set_gc_data(&mut self, reg: GCReg, v: u8) {
        self.gc_reg[reg as usize] = v;
    }

    //Update VGA memory (destination depends on register state SCReg::MapMask)
    pub fn set_mem(&mut self, offset: usize, v: u8) {
        //TODO set mem in all planes (depending on dest!, it's a mask)
        let dest = self.sc_reg[SCReg::MapMask as usize] - 1;
        self.mem[dest as usize][offset] = v;
    }

    //shortcut for setting a chunk of memory
    pub fn set_mem_chunk(&mut self, offset: usize, v: &Vec<u8>) {
        for (i, v) in v.iter().enumerate() {
            self.set_mem(offset + i, *v);
        }
    }
}
