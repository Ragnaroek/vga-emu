pub mod util;
pub mod input;
pub mod backend;
#[cfg(feature = "sdl")]
pub mod backend_sdl;
#[cfg(feature = "web")]
pub mod backend_web;

use std::sync::atomic::{AtomicU8, AtomicU16, Ordering, AtomicU64};
use std::sync::{RwLock, Arc, Mutex};
use input::InputMonitoring;

pub const TARGET_FRAME_RATE_MICRO: u128 = 1_000_000 / 70;
pub const VERTICAL_RESET_MICRO: u64 = 635;

const DEBUG_HEIGHT: usize = 20;
pub const FRAME_RATE_SAMPLES: usize = 100;
pub const PLANE_SIZE: usize = 0xFFFF; // 64KiB

pub struct Options
 {
    pub show_frame_rate: bool,
    pub start_addr_override: Option<usize>,
    pub input_monitoring: Option<Arc<Mutex<InputMonitoring>>>,
    /// This counter is increment on each frame
    pub frame_count: Arc<AtomicU64>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            show_frame_rate: false,
            //set in debug mode to ignore the start address set in the vga
            start_addr_override: None,
            input_monitoring: None,
            frame_count: Arc::new(AtomicU64::new(0)),
        }
    }
}

pub struct VGA {
    video_mode: AtomicU8,
    sc_reg: Vec<AtomicU8>,
    gc_reg: Vec<AtomicU8>,
    crt_reg: Vec<AtomicU8>,
    latch_reg: Vec<AtomicU8>,
    general_reg: Vec<AtomicU8>,
    attribute_reg: Vec<AtomicU8>,

    color_reg: Vec<AtomicU8>,
    color_write_reads: AtomicU16,
    palette_256: RwLock<[u32; 256]>,

    pub mem: Vec<Vec<AtomicU8>>,
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
        
        color_write_reads: AtomicU16::new(0),
        color_reg: init_atomic_u8_vec(4),
        palette_256: RwLock::new(init_default_256_palette()),

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

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum ColorReg {
    AddressWriteMode = 0x00,
    AddressReadMode = 0x01,
    Data = 0x02,
    State = 0x03,
}

impl VGA {
    pub fn start(self: Arc<Self>, options: Options) -> Result<(), String> {
        #[cfg(feature = "sdl")]
        return backend_sdl::start_sdl(self, options);
        #[cfg(feature = "web")]
        return backend_web::start_web(self, options);
    }

    /// Shows the full content of the VGA buffer as one big screen (for debugging) for
    /// the planar modes. width and height depends on your virtual screen size (640x819 if
    /// you did not change the default settings)
    pub fn start_debug_planar_mode(self: Arc<Self>, w: usize, h: usize, options: Options) -> Result<(), String> {
        let mut debug_options = options;
        debug_options.start_addr_override = Some(0);

        set_horizontal_display_end(&self, w as u32);
        set_vertical_display_end(&self, h as u32);

        self.start(debug_options)
}

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

    pub fn set_color_reg(&self, reg: ColorReg, v: u8) {        
        self.color_reg[reg as usize].swap(v, Ordering::AcqRel);
        if reg == ColorReg::Data {
            let writes = self.color_write_reads.fetch_add(1, Ordering::AcqRel);
            let ix = self.get_color_reg(ColorReg::AddressWriteMode) as usize;
            let color_part_shift = (2 - writes) * 8;           
            
            let mut table = self.palette_256.write().unwrap();
            table[ix] &= !((0xFF as u32) << color_part_shift);
            table[ix] |= ((v & 0x3F) as u32) << color_part_shift;
          
            if writes == 2 {
                self.color_reg[ColorReg::AddressWriteMode as usize].fetch_add(1, Ordering::AcqRel);
                self.color_write_reads.store(0, Ordering::Relaxed);
            }
        }
    }

    pub fn get_color_reg(&self, reg: ColorReg) -> u8 {
        if reg == ColorReg::Data {
            let reads = self.color_write_reads.fetch_add(1, Ordering::AcqRel);
            let ix = self.get_color_reg(ColorReg::AddressReadMode) as usize;
            let color_part_shift = (2 - reads) * 8;
            let color = self.get_color_palette_256(ix);
            let word = color & (0xFF as u32) << color_part_shift;      
             
            if reads == 2 {
                self.color_reg[ColorReg::AddressReadMode as usize].fetch_add(1, Ordering::AcqRel);
                self.color_write_reads.store(0, Ordering::Relaxed);
            }
            (word >> color_part_shift) as u8
        } else {
            self.color_reg[reg as usize].load(Ordering::Acquire)
        }
    }

    // Set through set_color_reg, this accesses the 256 palette directly
    pub fn get_color_palette_256(&self, ix: usize) -> u32 {
        let table = self.palette_256.read().unwrap();
        table[ix]
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

fn init_default_256_palette() -> [u32; 256] {
    //taken from https://commons.wikimedia.org/wiki/User:Psychonaut/ipalette.sh
    [
        0x000000, 0x0000AA, 0x00AA00, 0x00AAAA, 0xAA0000, 0xAA00AA, 0xAA5500, 0xAAAAAA, 0x555555,
        0x5555FF, 0x55FF55, 0x55FFFF, 0xFF5555, 0xFF55FF, 0xFFFF55, 0xFFFFFF, 0x000000, 0x101010,
        0x202020, 0x353535, 0x454545, 0x555555, 0x656565, 0x757575, 0x8A8A8A, 0x9A9A9A, 0xAAAAAA,
        0xBABABA, 0xCACACA, 0xDFDFDF, 0xEFEFEF, 0xFFFFFF, 0x0000FF, 0x4100FF, 0x8200FF, 0xBE00FF,
        0xFF00FF, 0xFF00BE, 0xFF0082, 0xFF0041, 0xFF0000, 0xFF4100, 0xFF8200, 0xFFBE00, 0xFFFF00,
        0xBEFF00, 0x82FF00, 0x41FF00, 0x00FF00, 0x00FF41, 0x00FF82, 0x00FFBE, 0x00FFFF, 0x00BEFF,
        0x0082FF, 0x0041FF, 0x8282FF, 0x9E82FF, 0xBE82FF, 0xDF82FF, 0xFF82FF, 0xFF82DF, 0xFF82BE,
        0xFF829E, 0xFF8282, 0xFF9E82, 0xFFBE82, 0xFFDF82, 0xFFFF82, 0xDFFF82, 0xBEFF82, 0x9EFF82,
        0x82FF82, 0x82FF9E, 0x82FFBE, 0x82FFDF, 0x82FFFF, 0x82DFFF, 0x82BEFF, 0x829EFF, 0xBABAFF,
        0xCABAFF, 0xDFBAFF, 0xEFBAFF, 0xFFBAFF, 0xFFBAEF, 0xFFBADF, 0xFFBACA, 0xFFBABA, 0xFFCABA,
        0xFFDFBA, 0xFFEFBA, 0xFFFFBA, 0xEFFFBA, 0xDFFFBA, 0xCAFFBA, 0xBAFFBA, 0xBAFFCA, 0xBAFFDF,
        0xBAFFEF, 0xBAFFFF, 0xBAEFFF, 0xBADFFF, 0xBACAFF, 0x000071, 0x1C0071, 0x390071, 0x550071,
        0x710071, 0x710055, 0x710039, 0x71001C, 0x710000, 0x711C00, 0x713900, 0x715500, 0x717100,
        0x557100, 0x397100, 0x1C7100, 0x007100, 0x00711C, 0x007139, 0x007155, 0x007171, 0x005571,
        0x003971, 0x001C71, 0x393971, 0x453971, 0x553971, 0x613971, 0x713971, 0x713961, 0x713955,
        0x713945, 0x713939, 0x714539, 0x715539, 0x716139, 0x717139, 0x617139, 0x557139, 0x457139,
        0x397139, 0x397145, 0x397155, 0x397161, 0x397171, 0x396171, 0x395571, 0x394571, 0x515171,
        0x595171, 0x615171, 0x695171, 0x715171, 0x715169, 0x715161, 0x715159, 0x715151, 0x715951,
        0x716151, 0x716951, 0x717151, 0x697151, 0x617151, 0x597151, 0x517151, 0x517159, 0x517161,
        0x517169, 0x517171, 0x516971, 0x516171, 0x515971, 0x000041, 0x100041, 0x200041, 0x310041,
        0x410041, 0x410031, 0x410020, 0x410010, 0x410000, 0x411000, 0x412000, 0x413100, 0x414100,
        0x314100, 0x204100, 0x104100, 0x004100, 0x004110, 0x004120, 0x004131, 0x004141, 0x003141,
        0x002041, 0x001041, 0x202041, 0x282041, 0x312041, 0x392041, 0x412041, 0x412039, 0x412031,
        0x412028, 0x412020, 0x412820, 0x413120, 0x413920, 0x414120, 0x394120, 0x314120, 0x284120,
        0x204120, 0x204128, 0x204131, 0x204139, 0x204141, 0x203941, 0x203141, 0x202841, 0x2D2D41,
        0x312D41, 0x352D41, 0x3D2D41, 0x412D41, 0x412D3D, 0x412D35, 0x412D31, 0x412D2D, 0x41312D,
        0x41352D, 0x413D2D, 0x41412D, 0x3D412D, 0x35412D, 0x31412D, 0x2D412D, 0x2D4131, 0x2D4135,
        0x2D413D, 0x2D4141, 0x2D3D41, 0x2D3541, 0x2D3141, 0x000000, 0x000000, 0x000000, 0x000000,
        0x000000, 0x000000, 0x000000, 0x00000,
    ]
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