pub mod screen;

const PLANE_SIZE: usize = 0xFFFF; //64KiB

pub struct VGA {
    sc_reg: [u8; 5],
    gc_reg: [u8; 9],
    crt_reg: [u8; 25],
    latch_reg: [u8; 4],
    pub mem: [[u8; PLANE_SIZE]; 4],
}

pub fn new() -> VGA {
    VGA {
        sc_reg: [0; 5],
        gc_reg: [0; 9],
        crt_reg: [0; 25],
        latch_reg: [0; 4],
        mem: [[0; PLANE_SIZE]; 4],
    }
}

//Sequence Controller Register
pub enum SCReg {
    Reset = 0x0,
    ClockingMode = 0x1,
    MapMask = 0x2,
    CharacterMapSelect = 0x3,
    SequencerMemoryMode = 0x4,
}

//Graphics Controller Register
pub enum GCReg {
    SetReset = 0x0,
    EnableSetReset = 0x1,
    ColorCompare = 0x2,
    DataRotate = 0x3,
    ReadMapSelect = 0x4,
    GraphicsMode = 0x5,
    MiscGraphics = 0x6,
    ColorDontCare = 0x7,
    BitMask = 0x8,
}

pub enum CRTReg {
    HorizontalTotal = 0x00,
    EndHorizontalDisplay = 0x01,
    StartHorizontalBlanking = 0x02,
    EndHorizontalBlanking = 0x03,
    StartHorizontalRetrace = 0x04,
    EndHorizontalRetrace = 0x05,
    VerticalTotal = 0x06,
    Overflow = 0x07,
    PresetRowScan = 0x08,
    MaximumScanLine = 0x09,
    CursorStart = 0x0A,
    CursorEnd = 0x0B,
    StartAdressHigh = 0x0C,
    StartAdressLow = 0x0D,
    CursorLocationHigh = 0x0E,
    CursorLocaionLow = 0x0F,
    VerticalRetraceStart = 0x10,
    VerticalRetraceEnd = 0x11,
    VerticalDisplayEnd = 0x12,
    Offset = 0x13,
    UnderlineLocation = 0x14,
    StartVerticalBlanking = 0x15,
    EndVerticalBlanking = 0x16,
    CRTCModeControl = 0x17,
    LineCompare = 0x18,
}

impl VGA {
    pub fn set_sc_data(&mut self, reg: SCReg, v: u8) {
        self.sc_reg[reg as usize] = v;
    }

    pub fn get_sc_data(&self, reg: SCReg) -> u8 {
        self.sc_reg[reg as usize]
    }

    pub fn set_gc_data(&mut self, reg: GCReg, v: u8) {
        self.gc_reg[reg as usize] = v;
    }

    pub fn get_gc_data(&self, reg: GCReg) -> u8 {
        self.gc_reg[reg as usize]
    }

    pub fn set_crt_data(&mut self, reg: CRTReg, v: u8) {
        self.crt_reg[reg as usize] = v;
    }

    pub fn get_crt_data(&self, reg: CRTReg) -> u8 {
        self.crt_reg[reg as usize]
    }

    /// Update VGA memory (destination depends on register state SCReg::MapMask)
    pub fn write_mem(&mut self, offset: usize, v_in: u8) {
        let dest = self.get_sc_data(SCReg::MapMask);
        let mut gc_mode = self.get_gc_data(GCReg::GraphicsMode);
        gc_mode &= 0x03;

        for i in 0..4 {
            if (dest & (1 << i)) != 0 {
                let v = if gc_mode == 0x01 {
                    self.latch_reg[i]
                } else {
                    v_in
                };
                self.mem[i][offset] = v;
            }
        }
    }

    pub fn read_mem(&mut self, offset: usize) -> u8 {
        let select = (self.get_gc_data(GCReg::ReadMapSelect) & 0x3) as usize;
        for i in 0..4 {
            self.latch_reg[i] = self.mem[i][offset];
        }
        return self.latch_reg[select];
    }

    /// Shortcut for setting a chunk of memory.
    pub fn write_mem_chunk(&mut self, offset: usize, v: &Vec<u8>) {
        for (i, v) in v.iter().enumerate() {
            self.write_mem(offset + i, *v);
        }
    }
}
