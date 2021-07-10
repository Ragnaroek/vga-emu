extern crate vga;

use vga::{GCReg, SCReg};

#[test]
fn test_write_read_mem_mode_0() {
    let vga = vga::new(0x10);
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
    let vga = vga::new(0x10);
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
fn test_read_mem() {}
