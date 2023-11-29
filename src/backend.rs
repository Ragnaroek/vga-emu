/// Contains common functionality shared across all backend implementations
use crate::{CRTReg, VGA, AttributeReg, Options};

#[derive(Debug)]
pub struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

pub fn rgb(r: u8, g: u8, b: u8) -> RGB {
    RGB{r, g, b}
}

pub trait PixelBuffer {
    const PIXEL_WIDTH : usize;
    fn set_rgb(&mut self, offset: usize, r: u8, g: u8, b: u8);
}

/// pitch = length of one row in bytes
pub fn render_planar<T: PixelBuffer + ?Sized>(
    vga: &VGA, mem_offset_p: usize, offset_delta: usize, h: usize, 
    buffer: &mut T, pitch: usize,
) {
    let mut x: usize = 0;
    let mut y: usize = 0;
    let mut mem_offset = mem_offset_p;
    let max_scan = (vga.get_crt_data(CRTReg::MaximumScanLine) & 0x1F) as usize + 1;
    let w_bytes = vga.get_crt_data(CRTReg::HorizontalDisplayEnd) as usize + 2; //+1 for exclusive intervall, +1 for "overshot" with potential hpan

    for _ in 0..(h / max_scan) {
        for _ in 0..max_scan {
            let hpan = vga.get_attribute_reg(AttributeReg::HorizontalPixelPanning) & 0xF;
            for mem_byte in 0..w_bytes {
                let v0 = vga.raw_read_mem(0, mem_offset + mem_byte);
                let v1 = vga.raw_read_mem(1, mem_offset + mem_byte);
                let v2 = vga.raw_read_mem(2, mem_offset + mem_byte);
                let v3 = vga.raw_read_mem(3, mem_offset + mem_byte);

                let start = if mem_byte == 0 { hpan } else { 0 };
                let end = if mem_byte == w_bytes - 1 { hpan } else { 8 };
                for b in start..end {
                    let bx = (1 << (7 - b)) as u8;
                    let mut pixel = bit_x(v0, bx, 0);
                    pixel |= bit_x(v1, bx, 1);
                    pixel |= bit_x(v2, bx, 2);
                    pixel |= bit_x(v3, bx, 3);

                    let color = default_16_color(pixel);
                    let offset = y * pitch + x * T::PIXEL_WIDTH;
                    buffer.set_rgb(offset, color.r, color.g, color.b);
 
                    x += 1;
                }
            }
            x = 0;
            y += 1;
        }
        mem_offset += offset_delta * 2;
    }
}

pub fn render_linear<T: PixelBuffer + ?Sized>(
    vga: &VGA, mem_offset_p: usize, offset_delta: usize, h: usize, v_stretch: usize,
    buffer: &mut T,
) {
    let mut mem_offset = mem_offset_p;
    let max_scan = (vga.get_crt_data(CRTReg::MaximumScanLine) & 0x1F) as usize + 1;
    let w_bytes = vga.get_crt_data(CRTReg::HorizontalDisplayEnd) as usize + 1;

    let mut buffer_offset = 0;
    for _ in 0..((h / max_scan) as usize) {
        for _ in 0..max_scan {
            for x_byte in 0..w_bytes {
                for p in 0..4 {
                    let v = vga.raw_read_mem(p, mem_offset + x_byte);
                    let color = vga.get_color_palette_256(v as usize);
                    for _ in 0..v_stretch {
                        // each color part (RGB) contains the high-order 6 bit values. 
                        // To get a "real" RGB value for display the value have to shifted
                        // by 2 bits (otherwise the color will be dimmed)
                        buffer.set_rgb(buffer_offset, ((color & 0xFF0000) >> 14) as u8, ((color & 0x00FF00) >> 6) as u8, ((color & 0x0000FF) << 2) as u8);
                        buffer_offset += T::PIXEL_WIDTH;
                    }
                }
            }
        }
        mem_offset += offset_delta * 2;
    }
}

fn bit_x(v: u8, v_ix: u8, dst_ix: u8) -> u8 {
    if v & v_ix != 0 {
        1 << dst_ix
    } else {
        0
    }
}

fn default_16_color(v: u8) -> RGB {
    //source: https://wasteland.fandom.com/wiki/EGA_Colour_Palette
    match v {
        0x00 => rgb(0x0, 0x0, 0x0),
        0x01 => rgb(0x0, 0x0, 0xA8),
        0x02 => rgb(0x0, 0xA8, 0x0),
        0x03 => rgb(0x0, 0xA8, 0xA8),
        0x04 => rgb(0xA8, 0x0, 0x0),
        0x05 => rgb(0xA8, 0x0, 0xA8),
        0x06 => rgb(0xA8, 0x54, 0x00),
        0x07 => rgb(0xA8, 0xA8, 0xA8),
        0x08 => rgb(0x54, 0x54, 0x54),
        0x09 => rgb(0x54, 0x54, 0xFE),
        0x0A => rgb(0x54, 0xFE, 0x54),
        0x0B => rgb(0x54, 0xFE, 0xFE),
        0x0C => rgb(0xFE, 0x54, 0x54),
        0x0D => rgb(0xFE, 0x54, 0xFE),
        0x0E => rgb(0xFE, 0xFE, 0x54),
        0x0F => rgb(0xFE, 0xFE, 0xFE),
        _ => panic!("wrong color index: {}", v),
    }
}

pub fn is_linear(vmode: u8) -> bool {
    vmode == 0x13
}

pub fn mem_offset(vga: &VGA, options: &Options) -> usize {
    if let Some(over) = options.start_addr_override {
        return over;
    }
    let low = vga.get_crt_data(CRTReg::StartAdressLow) as u16;
    let mut addr = vga.get_crt_data(CRTReg::StartAdressHigh) as u16;
    addr <<= 8;
    addr |= low;
    addr as usize
}