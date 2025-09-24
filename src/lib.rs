#[cfg(test)]
#[path = "./lib_test.rs"]
mod lib_test;

pub mod backend;
#[cfg(feature = "sdl3")]
pub mod backend_sdl3;
#[cfg(feature = "test")]
pub mod backend_test;
#[cfg(feature = "web")]
pub mod backend_web;
pub mod input;
pub mod util;

#[cfg(feature = "sdl3")]
pub use backend_sdl3::RenderContext;
#[cfg(feature = "test")]
pub use backend_test::RenderContext;
#[cfg(feature = "web")]
pub use backend_web::RenderContext;

#[cfg(feature = "tracing")]
use tracing::instrument;

use std::sync::atomic::{AtomicU8, AtomicU16, Ordering};
use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard};

use input::InputMonitoring;
use util::{get_height_regs, get_width_regs};

pub const VERTICAL_RESET_MICRO: u64 = 635;

pub const PLANE_SIZE: usize = 0xFFFF; // 64KiB

pub struct VGARegs {
    sc_reg: Vec<AtomicU8>,
    gc_reg: Vec<AtomicU8>,
    crt_reg: Vec<AtomicU8>,
    latch_reg: Vec<AtomicU8>,
    general_reg: Vec<AtomicU8>,
    attribute_reg: Vec<AtomicU8>,
    color_reg: Vec<AtomicU8>,

    color_write_reads: AtomicU16,
    video_mode: AtomicU8,
}

pub struct VGA {
    vga_emu: VGAEmu,
    rc: RenderContext,
}

pub struct VGAEmu {
    regs: VGARegs,
    palette_256: RwLock<[u32; 256]>,
    pub mem: Mutex<Vec<Vec<u8>>>,
    pub start_addr_override: Option<usize>,
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

impl VGARegs {
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
}

pub struct VGABuilder {
    video_mode: u8,
    fullscreen: bool,
    simulate_vertical_reset: bool,
    start_addr_override: Option<usize>,
}

impl VGABuilder {
    pub fn new() -> VGABuilder {
        VGABuilder {
            video_mode: 0x10,
            fullscreen: true,
            simulate_vertical_reset: false,
            start_addr_override: None,
        }
    }

    pub fn video_mode(mut self, mode: u8) -> VGABuilder {
        self.video_mode = mode;
        self
    }

    pub fn fullscreen(mut self, fullscreen: bool) -> VGABuilder {
        self.fullscreen = fullscreen;
        self
    }

    /// If activated this will simulate the vertical reset. Some
    /// programs may use this and observe the corresponding register
    /// status changes.
    /// By default this is not enabled.
    pub fn simulate_vertical_reset(mut self) -> VGABuilder {
        self.simulate_vertical_reset = true;
        self
    }

    pub fn start_addr_override(mut self, over: usize) -> VGABuilder {
        self.start_addr_override = Some(over);
        self
    }

    /// Constructs a VGA depending on the compile options (see
    /// features list for available options)
    pub fn build(self) -> Result<VGA, String> {
        VGA::setup(self)
    }
}

impl VGA {
    pub fn setup(builder: VGABuilder) -> Result<VGA, String> {
        let vga_emu = VGAEmu::new(&builder);

        let width = get_width_regs(&vga_emu.regs);
        let height = get_height_regs(&vga_emu.regs);
        let rc = RenderContext::init(width as usize, height as usize, builder)?;

        Ok(VGA { vga_emu, rc })
    }

    pub fn draw_frame(&mut self) -> bool {
        self.rc.draw_frame(&self.vga_emu)
    }

    pub fn set_sc_data(&self, reg: SCReg, v: u8) {
        self.vga_emu.regs.set_sc_data(reg, v);
    }

    pub fn get_sc_data(&self, reg: SCReg) -> u8 {
        self.vga_emu.regs.get_sc_data(reg)
    }

    pub fn set_gc_data(&self, reg: GCReg, v: u8) {
        self.vga_emu.regs.set_gc_data(reg, v);
    }

    pub fn get_gc_data(&self, reg: GCReg) -> u8 {
        self.vga_emu.regs.get_gc_data(reg)
    }

    #[cfg_attr(feature = "tracing", instrument(skip_all))]
    pub fn set_crt_data(&self, reg: CRTReg, v: u8) {
        self.vga_emu.regs.set_crt_data(reg, v);
    }

    pub fn get_crt_data(&self, reg: CRTReg) -> u8 {
        self.vga_emu.regs.get_crt_data(reg)
    }

    pub fn set_general_reg(&self, reg: GeneralReg, v: u8) {
        self.vga_emu.regs.set_general_reg(reg, v);
    }

    pub fn get_general_reg(&self, reg: GeneralReg) -> u8 {
        self.vga_emu.regs.get_general_reg(reg)
    }

    pub fn set_attribute_reg(&self, reg: AttributeReg, v: u8) {
        self.vga_emu.regs.set_attribute_reg(reg, v);
    }

    pub fn get_attribute_reg(&self, reg: AttributeReg) -> u8 {
        self.vga_emu.regs.get_attribute_reg(reg)
    }

    pub fn get_video_mode(&self) -> u8 {
        self.vga_emu.get_video_mode()
    }

    pub fn set_color_reg(&self, reg: ColorReg, v: u8) {
        self.vga_emu.set_color_reg(reg, v)
    }

    pub fn get_color_reg(&self, reg: ColorReg) -> u8 {
        self.vga_emu.get_color_reg(reg)
    }

    pub fn get_color_palette_256_value(&self, ix: usize) -> u32 {
        self.vga_emu.get_color_palette_256_value(ix)
    }

    pub fn write_mem(&self, offset: usize, v_in: u8) {
        self.vga_emu.write_mem(offset, v_in)
    }

    pub fn read_mem(&self, offset: usize) -> u8 {
        self.vga_emu.read_mem(offset)
    }

    pub fn raw_read_mem(&self, plane: usize, offset: usize) -> u8 {
        self.vga_emu.raw_read_mem(plane, offset)
    }

    //useful for testing, set the memory in a given plane
    pub fn raw_write_mem(&self, plane: usize, offset: usize, v: u8) {
        self.vga_emu.raw_write_mem(plane, offset, v)
    }

    pub fn write_mem_chunk(&self, offset: usize, v: &[u8]) {
        self.vga_emu.write_mem_chunk(offset, v)
    }

    pub fn input_monitoring(&mut self) -> &mut InputMonitoring {
        self.rc.input_monitoring()
    }
}

impl VGAEmu {
    pub fn new(builder: &VGABuilder) -> VGAEmu {
        let mem = vec![
            vec![0; PLANE_SIZE],
            vec![0; PLANE_SIZE],
            vec![0; PLANE_SIZE],
            vec![0; PLANE_SIZE],
        ];

        let regs = VGARegs {
            sc_reg: init_atomic_u8_vec(5),
            gc_reg: init_atomic_u8_vec(9),
            crt_reg: init_atomic_u8_vec(25),
            latch_reg: init_atomic_u8_vec(4),
            general_reg: init_atomic_u8_vec(4),
            attribute_reg: init_atomic_u8_vec(21),

            video_mode: AtomicU8::new(builder.video_mode),
            color_write_reads: AtomicU16::new(0),
            color_reg: init_atomic_u8_vec(4),
        };

        setup_defaults(&regs);

        match builder.video_mode {
            0x10 => setup_mode_10(&regs),
            0x13 => setup_mode_13(&regs),
            _ => panic!(
                "video mode {:x}h not yet implemented",
                regs.get_video_mode()
            ),
        }

        VGAEmu {
            regs,
            palette_256: RwLock::new(init_default_256_palette()),
            mem: Mutex::new(mem),
            start_addr_override: builder.start_addr_override,
        }
    }

    pub fn set_color_reg(&self, reg: ColorReg, v: u8) {
        self.regs.color_reg[reg as usize].swap(v, Ordering::AcqRel);
        if reg == ColorReg::Data {
            let writes = self.regs.color_write_reads.fetch_add(1, Ordering::AcqRel);
            let ix = self.get_color_reg(ColorReg::AddressWriteMode) as usize;
            let color_part_shift = (2 - writes) * 8;

            let mut table = self.palette_256.write().unwrap();
            table[ix] &= !((0xFF as u32) << color_part_shift);
            table[ix] |= ((v & 0x3F) as u32) << color_part_shift;

            if writes == 2 {
                self.regs.color_reg[ColorReg::AddressWriteMode as usize]
                    .fetch_add(1, Ordering::AcqRel);
                self.regs.color_write_reads.store(0, Ordering::Relaxed);
            }
        }
    }

    pub fn get_color_reg(&self, reg: ColorReg) -> u8 {
        if reg == ColorReg::Data {
            let reads = self.regs.color_write_reads.fetch_add(1, Ordering::AcqRel);
            let ix = self.get_color_reg(ColorReg::AddressReadMode) as usize;
            let color_part_shift = (2 - reads) * 8;
            let color = self.get_color_palette_256_value(ix);
            let word = color & (0xFF as u32) << color_part_shift;

            if reads == 2 {
                self.regs.color_reg[ColorReg::AddressReadMode as usize]
                    .fetch_add(1, Ordering::AcqRel);
                self.regs.color_write_reads.store(0, Ordering::Relaxed);
            }
            (word >> color_part_shift) as u8
        } else {
            self.regs.color_reg[reg as usize].load(Ordering::Acquire)
        }
    }

    // Set through set_color_reg, this accesses the 256 palette directly
    pub fn get_color_palette_256_value(&self, ix: usize) -> u32 {
        let table = self.palette_256.read().unwrap();
        table[ix]
    }

    pub fn get_palette_256(&self) -> RwLockReadGuard<'_, [u32; 256]> {
        self.palette_256.read().unwrap()
    }

    pub fn get_video_mode(&self) -> u8 {
        self.regs.get_video_mode()
    }

    /// Update VGA memory (destination depends on register state SCReg::MapMask)
    pub fn write_mem(&self, offset: usize, v_in: u8) {
        let mem_mode = self.regs.get_sc_data(SCReg::MemoryMode);
        let dest = if mem_mode & 0x08 != 0 {
            //if chain4 is enabled write to all planes
            0x0F
        } else if mem_mode & 0x04 == 0 {
            //odd/even enabled, determine plane on odd/even address
            if offset % 2 == 0 { 0x05 } else { 0x0A }
        } else {
            self.regs.get_sc_data(SCReg::MapMask)
        };

        let mut gc_mode = self.regs.get_gc_data(GCReg::GraphicsMode);
        let bit_mask = self.regs.get_gc_data(GCReg::BitMask);
        gc_mode &= 0x03;

        let mut mem_lock = self.mem.lock().unwrap();
        for i in 0..4 {
            if (dest & (1 << i)) != 0 {
                let v = if gc_mode == 0x01 {
                    self.regs.latch_reg[i].load(Ordering::Acquire)
                } else {
                    let v_latch = self.regs.latch_reg[i].load(Ordering::Acquire);
                    v_in & bit_mask | (v_latch & !bit_mask)
                };
                mem_lock[i][offset] = v;
            }
        }
    }

    //useful for testing, inspect the memory for a given plane
    pub fn raw_read_mem(&self, plane: usize, offset: usize) -> u8 {
        let lock_mem = self.mem.lock().unwrap();
        lock_mem[plane][offset]
    }

    //useful for testing, set the memory in a given plane
    pub fn raw_write_mem(&self, plane: usize, offset: usize, v: u8) {
        let mut lock_mem = self.mem.lock().unwrap();
        lock_mem[plane][offset] = v;
    }

    /// Shortcut for setting a chunk of memory.
    pub fn write_mem_chunk(&self, offset: usize, v: &[u8]) {
        for (i, v) in v.iter().enumerate() {
            self.write_mem(offset + i, *v);
        }
    }

    pub fn read_mem(&self, offset: usize) -> u8 {
        let mem_mode = self.regs.get_sc_data(SCReg::MemoryMode);
        let select = if mem_mode & 0x08 != 0 {
            //if chain4 is enabled, read from the plan determined by the offsets lower 2 bits
            (offset & 0x03) as usize
        } else {
            (self.regs.get_gc_data(GCReg::ReadMapSelect) & 0x3) as usize
        };
        let lock_mem = self.mem.lock().unwrap();
        for i in 0..4 {
            self.regs.latch_reg[i].swap(lock_mem[i][offset], Ordering::AcqRel);
        }
        self.regs.latch_reg[select].load(Ordering::Acquire)
    }

    pub fn mem_lock(&self) -> MutexGuard<'_, Vec<Vec<u8>>> {
        self.mem.lock().unwrap()
    }

    pub fn raw_read_mem_with_lock(
        &self, lock: &MutexGuard<Vec<Vec<u8>>>, plane: usize, offset: usize,
    ) -> u8 {
        lock[plane][offset]
    }

    pub fn mem_offset(&self) -> usize {
        if let Some(over) = self.start_addr_override {
            return over;
        }

        let low = self.regs.get_crt_data(CRTReg::StartAdressLow) as u16;
        let mut addr = self.regs.get_crt_data(CRTReg::StartAdressHigh) as u16;
        addr <<= 8;
        addr |= low;
        addr as usize
    }
}

fn setup_defaults(regs: &VGARegs) {
    regs.set_crt_data(CRTReg::Offset, 40);
    regs.set_gc_data(GCReg::BitMask, 0xFF);
}

fn setup_mode_10(regs: &VGARegs) {
    regs.set_sc_data(SCReg::MemoryMode, 0x04); //disable chain 4, disable odd/even
    regs.set_crt_data(CRTReg::MaximumScanLine, 0x00);
    set_regs_horizontal_display_end(regs, 640);
    set_regs_vertical_display_end(regs, 350);
}

fn setup_mode_13(regs: &VGARegs) {
    regs.set_sc_data(SCReg::MemoryMode, 0x08); //enable chain 4, enable odd/even
    regs.set_crt_data(CRTReg::MaximumScanLine, 0x01);
    set_regs_horizontal_display_end(regs, 640);
    set_regs_vertical_display_end(regs, 400);
}

fn init_atomic_u8_vec(len: usize) -> Vec<AtomicU8> {
    let mut vec = Vec::with_capacity(len);
    for _ in 0..len {
        vec.push(AtomicU8::new(0));
    }
    vec
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

fn set_regs_horizontal_display_end(regs: &VGARegs, width: u32) {
    regs.set_crt_data(CRTReg::HorizontalDisplayEnd, ((width - 1) / 8) as u8);
}

pub fn set_horizontal_display_end(vga: &VGA, width: u32) {
    vga.vga_emu
        .regs
        .set_crt_data(CRTReg::HorizontalDisplayEnd, ((width - 1) / 8) as u8);
}

fn set_regs_vertical_display_end(regs: &VGARegs, height: u32) {
    let h = height - 1;
    regs.set_crt_data(CRTReg::VerticalDisplayEnd, h as u8);
    let bit_8 = ((h & 0x100) >> 8) as u8;
    let bit_9 = ((h & 0x200) >> 9) as u8;
    let overflow = bit_9 << 6 | bit_8 << 1;
    regs.set_crt_data(CRTReg::Overflow, overflow);
}

pub fn set_vertical_display_end(vga: &VGA, height: u32) {
    set_regs_vertical_display_end(&vga.vga_emu.regs, height);
}
