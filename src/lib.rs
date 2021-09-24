pub mod screen;
pub mod util;

use std::sync::atomic::{AtomicU8, Ordering};

pub const PLANE_SIZE: usize = 0xFFFF; //64KiB

pub struct VGA {
    video_mode: AtomicU8,
    sc_reg: Vec<AtomicU8>,
    gc_reg: Vec<AtomicU8>,
    crt_reg: Vec<AtomicU8>,
    latch_reg: Vec<AtomicU8>,
    general_reg: Vec<AtomicU8>,
    attribute_reg: Vec<AtomicU8>,
    mem: Vec<Vec<AtomicU8>>,
}

pub fn new(video_mode: u8) -> VGA {
    let mem = vec![
        init_atomic_u8_vec(PLANE_SIZE),
        init_atomic_u8_vec(PLANE_SIZE),
        init_atomic_u8_vec(PLANE_SIZE),
        init_atomic_u8_vec(PLANE_SIZE),
    ];

    let vga = VGA {
        video_mode: AtomicU8::new(video_mode),
        sc_reg: init_atomic_u8_vec(5),
        gc_reg: init_atomic_u8_vec(9),
        crt_reg:  init_atomic_u8_vec(25),
        latch_reg: init_atomic_u8_vec(4),
        general_reg: init_atomic_u8_vec(4),
        attribute_reg: init_atomic_u8_vec(21),
        mem,
    };

    setup_defaults(&vga);

    match video_mode {
        0x10 => setup_mode_10(&vga),
        0x13 => setup_mode_13(&vga),
        _ => panic!("video mode {:x}h not yet implemented", vga.get_video_mode()),
    }    

    vga
}

fn setup_defaults(vga: &VGA) {
    vga.set_crt_data(CRTReg::Offset, 40);

    vga.set_gc_data(GCReg::BitMask, 0xFF);
}

fn setup_mode_10(vga: &VGA) {
    vga.set_sc_data(SCReg::MemoryMode, 0x04); //disable chain 4, disable odd/even
    vga.set_crt_data(CRTReg::MaximumScanLine, 0x00);
    set_horizontal_display_end(vga, 640);
    set_vertical_display_end(vga, 350);
}

fn setup_mode_13(vga: &VGA) {
    vga.set_sc_data(SCReg::MemoryMode, 0x08); //enable chain 4, enable odd/even
    vga.set_crt_data(CRTReg::MaximumScanLine, 0x01);
    set_horizontal_display_end(vga, 640);
    set_vertical_display_end(vga, 400);
}

fn init_atomic_u8_vec(len: usize) -> Vec<AtomicU8> {
    let mut vec = Vec::with_capacity(len);
    for _ in 0..len {
        vec.push(AtomicU8::new(0));
    }
    vec
}

//Sequence Controller Register
pub enum SCReg {
    Reset = 0x0,
    ClockingMode = 0x1,
    MapMask = 0x2,
    CharacterMapSelect = 0x3,
    MemoryMode = 0x4,
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

//CRT Controller Registers
pub enum CRTReg {
    HorizontalTotal = 0x00,
    HorizontalDisplayEnd = 0x01,
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

pub enum GeneralReg {
    MiscOutput = 0x00,
    FeatureContorl = 0x01,
    InputStatus0 = 0x02,
    InputStatus1 = 0x03,
}

pub enum AttributeReg {
    Palette0 = 0x00,
    Palette1 = 0x01,
    Palette2 = 0x02,
    Palette3 = 0x03,
    Palette4 = 0x04,
    Palette5 = 0x05,
    Palette6 = 0x06,
    Palette7 = 0x07,
    Palette8 = 0x08,
    Palette9 = 0x09,
    Palette10 = 0x0A,
    Palette11 = 0x0B,
    Palette12 = 0x0C,
    Palette13 = 0x0D,
    Palette14 = 0x0E,
    Palette15 = 0x0F,
    ModeControl = 0x10,
    OverscanColor = 0x11,
    ColorPlaneEnable = 0x12,
    HorizontalPixelPanning = 0x13,
    ColorPlaneEnableVGA = 0x14,
}

impl VGA {
    pub fn set_sc_data(&self, reg: SCReg, v: u8) {
        self.sc_reg[reg as usize].swap(v, Ordering::AcqRel);
    }

    pub fn get_sc_data(&self, reg: SCReg) -> u8 {
        self.sc_reg[reg as usize].load(Ordering::Acquire)
    }

    pub fn set_gc_data(&self, reg: GCReg, v: u8) {
        self.gc_reg[reg as usize].swap(v, Ordering::AcqRel);
    }

    pub fn get_gc_data(&self, reg: GCReg) -> u8 {
        self.gc_reg[reg as usize].load(Ordering::Acquire)
    }

    pub fn set_crt_data(&self, reg: CRTReg, v: u8) {
        self.crt_reg[reg as usize].swap(v, Ordering::AcqRel);
    }

    pub fn get_crt_data(&self, reg: CRTReg) -> u8 {
        self.crt_reg[reg as usize].load(Ordering::Acquire)
    }

    pub fn set_general_reg(&self, reg: GeneralReg, v: u8) {
        self.general_reg[reg as usize].swap(v, Ordering::AcqRel);
    }

    pub fn get_general_reg(&self, reg: GeneralReg) -> u8 {
        self.general_reg[reg as usize].load(Ordering::Acquire)
    }

    pub fn set_attribute_reg(&self, reg: AttributeReg, v: u8) {
        self.attribute_reg[reg as usize].swap(v, Ordering::AcqRel);
    }

    pub fn get_attribute_reg(&self, reg: AttributeReg) -> u8 {
        self.attribute_reg[reg as usize].load(Ordering::Acquire)
    }

    pub fn get_video_mode(&self) -> u8 {
        self.video_mode.load(Ordering::Acquire)
    }

    /// Update VGA memory (destination depends on register state SCReg::MapMask)
    pub fn write_mem(&self, offset: usize, v_in: u8) {
        let mem_mode = self.get_sc_data(SCReg::MemoryMode);
        let dest = if mem_mode & 0x08 != 0 {
            //if chain4 is enabled write to all planes
            0x0F
        } else if mem_mode & 0x04 == 0 {
            //odd/even enabled, determine plane on odd/even address
            if offset % 2 == 0 {
                0x05
            } else {
                0x0A
            }
        } else {
            self.get_sc_data(SCReg::MapMask)
        };

        let mut gc_mode = self.get_gc_data(GCReg::GraphicsMode);
        let bit_mask = self.get_gc_data(GCReg::BitMask);
        gc_mode &= 0x03;

        for i in 0..4 {
            if (dest & (1 << i)) != 0 {
                let v = if gc_mode == 0x01 {
                    self.latch_reg[i].load(Ordering::Acquire)
                } else {
                    let v_latch = self.latch_reg[i].load(Ordering::Acquire);
                    v_in & bit_mask | (v_latch & !bit_mask)
                };
                self.mem[i][offset].swap(v, Ordering::Relaxed);
            }
        }
    }

    /// Shortcut for setting a chunk of memory.
    pub fn write_mem_chunk(&self, offset: usize, v: &[u8]) {
        for (i, v) in v.iter().enumerate() {
            self.write_mem(offset + i, *v);
        }
    }

    pub fn read_mem(&self, offset: usize) -> u8 {
        let mem_mode = self.get_sc_data(SCReg::MemoryMode);
        let select = if mem_mode & 0x08 != 0 {
            //if chain4 is enabled, read from the plan determined by the offsets lower 2 bits
            (offset & 0x03) as usize
        } else {
            (self.get_gc_data(GCReg::ReadMapSelect) & 0x3) as usize
        };
        for i in 0..4 {
            self.latch_reg[i].swap(self.mem[i][offset].load(Ordering::Acquire), Ordering::AcqRel);
        }
        self.latch_reg[select].load(Ordering::Acquire)
    }

    //useful for testing, inspect the memory for a given plane
    pub fn raw_read_mem(&self, plane: usize, offset: usize) -> u8 {
        self.mem[plane][offset].load(Ordering::Relaxed)
    }

    //useful for testing, set the memory in a given plane
    pub fn raw_write_mem(&self, plane: usize, offset: usize, v: u8) {
        self.mem[plane][offset].swap(v, Ordering::AcqRel);
    }
}

pub fn is_linear(vmode: u8) -> bool {
    vmode == 0x13
}

 //convenience functions

pub fn set_horizontal_display_end(vga: &VGA, width: u32) {
    vga.set_crt_data(CRTReg::HorizontalDisplayEnd, ((width-1)/8) as u8);
}

pub fn set_vertical_display_end(vga: &VGA, height: u32) {
    let h = height-1;
    vga.set_crt_data(CRTReg::VerticalDisplayEnd, h as u8);
    let bit_8 = ((h & 0x100) >> 8) as u8;
    let bit_9 = ((h & 0x200) >> 9) as u8;
    let overflow = bit_9 << 6 | bit_8 << 1;
    vga.set_crt_data(CRTReg::Overflow, overflow); 
}