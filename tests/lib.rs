extern crate vga;

use vga::util::{get_height, get_width};
use vga::{
    set_horizontal_display_end, set_vertical_display_end, ColorReg, GCReg, SCReg, PLANE_SIZE, VGA,
};

#[test]
fn test_write_read_mem_mode_0() {
    let vga = VGA::setup_no_backend(0x10);
    vga.write_mem(666, 42);
    assert_eq!(vga.read_mem(666), 0);

    vga.set_sc_data(SCReg::MapMask, 0x01);
    vga.write_mem(666, 42);
    assert_eq!(vga.read_mem(666), 42);

    vga.set_sc_data(SCReg::MapMask, 0x08);
    vga.write_mem(666, 32);
    assert_eq!(vga.read_mem(666), 42);
    vga.set_gc_data(GCReg::ReadMapSelect, 0x03);
    assert_eq!(vga.read_mem(666), 32);

    vga.set_sc_data(SCReg::MapMask, 0x0F);
    vga.write_mem(666, 112);
    for i in 0..4 {
        vga.set_gc_data(GCReg::ReadMapSelect, i);
        assert_eq!(vga.read_mem(666), 112);
    }
}

#[test]
fn test_write_read_mem_mode_1() {
    let vga = VGA::setup_no_backend(0x10);
    vga.set_sc_data(SCReg::MapMask, 0x0F);
    vga.write_mem(666, 66);
    for i in 0..4 {
        vga.set_gc_data(GCReg::ReadMapSelect, i);
        assert_eq!(vga.read_mem(666), 66);
    }

    let mut gc_mode = vga.get_gc_data(GCReg::GraphicsMode);
    gc_mode &= 0xFC;
    gc_mode |= 0x01;
    vga.set_gc_data(GCReg::GraphicsMode, gc_mode);
    vga.write_mem(888, 0xFF); //value doesn't matter
    for i in 0..4 {
        vga.set_gc_data(GCReg::ReadMapSelect, i);
        assert_eq!(vga.read_mem(888), 66);
    }
}

#[test]
fn test_write_read_chain_4() {
    let vga = VGA::setup_no_backend(0x13); //mode 13 has chain4 enabled (also odd/even is enabled but this is ignored if chain4 is enabled)

    for i in 0..PLANE_SIZE {
        vga.write_mem(i, i as u8);
        for p in 0..4 {
            //writes to all planes
            assert_eq!(vga.raw_read_mem(p, i), i as u8);
        }
    }

    for i in 0..PLANE_SIZE {
        let plane_ix = (i & 0x3) as usize;
        for p in 0..4 {
            //reset all memory from other planes to check that we read from the right plane
            if p != plane_ix {
                vga.raw_write_mem(p, i, 0);
            }
        }
        let v = vga.read_mem(i);
        assert_eq!(v, i as u8);
    }
}

#[test]
fn test_write_read_odd_even() {
    let vga = VGA::setup_no_backend(0x13); //mode 13 has odd/even enabled
    vga.set_sc_data(
        SCReg::MemoryMode,
        vga.get_sc_data(SCReg::MemoryMode) & !0x08,
    ); //disable chain4 (otherwise odd/even is not enabled)

    for i in 0..PLANE_SIZE {
        vga.write_mem(i, i as u8);
        for p in 0..4 {
            //writes
            let expected = if i % 2 == p % 2 { i as u8 } else { 0 };
            assert_eq!(vga.raw_read_mem(p, i), expected);
        }
    }
}

#[test]
fn test_bit_mask() {
    let vga = VGA::setup_no_backend(0x13); //mode 13 has odd/even enabled
    vga.set_sc_data(SCReg::MapMask, 0xFF);
    vga.write_mem(666, 0xFF);
    for i in 0..4 {
        assert_eq!(vga.raw_read_mem(i, 666), 0xFF);
    }

    vga.set_gc_data(GCReg::BitMask, 0x0F);
    vga.write_mem(666, 0xFF);
    for i in 0..4 {
        assert_eq!(vga.raw_read_mem(i, 666), 0x0F);
    }

    vga.set_gc_data(GCReg::BitMask, 0xFF);
    vga.write_mem(600, 0b10101111);
    vga.read_mem(600); //latch memory
    vga.set_gc_data(GCReg::BitMask, 0x0F);
    vga.write_mem(666, 0);
    for i in 0..4 {
        assert_eq!(vga.raw_read_mem(i, 666), 0b10100000);
    }
}

#[test]
fn test_set_and_get_horizontal_display_end() {
    let vga = VGA::setup_no_backend(0x10);
    set_horizontal_display_end(&vga, 640);
    assert_eq!(get_width(&vga), 640);
}

#[test]
fn test_set_and_get_vertical_display_end() {
    let vga = VGA::setup_no_backend(0x10);
    set_vertical_display_end(&vga, 400);
    assert_eq!(get_height(&vga), 400);

    set_vertical_display_end(&vga, 1024);
    assert_eq!(get_height(&vga), 1024);
}

#[test]
fn test_set_color() {
    let vga = VGA::setup_no_backend(0x10);
    vga.set_color_reg(ColorReg::AddressWriteMode, 0);

    for i in 0..3 {
        assert_eq!(vga.get_color_reg(ColorReg::AddressWriteMode), 0);
        vga.set_color_reg(ColorReg::Data, 0x3F - i);
    }

    assert_eq!(vga.get_color_reg(ColorReg::AddressWriteMode), 1);
    assert_eq!(vga.get_color_palette_256(0), 0x3F3E3D);

    //TODO Write colors here
    //check auto-increment
    //check state register set to 0b11
}
