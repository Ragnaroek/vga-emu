extern crate vga;

use vga::{GCReg, SCReg, PLANE_SIZE};

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
fn test_write_read_chain_4() {
    let vga = vga::new(0x13); //mode 13 has chain4 enabled (also odd/even is enabled but this is ignored if chain4 is enabled)

    for i in 0..PLANE_SIZE {
        vga.write_mem(i, i as u8);
        for p in 0..4 { //writes to all planes
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
    let vga = vga::new(0x13); //mode 13 has odd/even enabled
    vga.set_sc_data(SCReg::MemoryMode, vga.get_sc_data(SCReg::MemoryMode) & !0x08); //disable chain4 (otherwise odd/even is not enabled)

    for i in 0..PLANE_SIZE {
        vga.write_mem(i, i as u8);
        for p in 0..4 { //writes
            let expected = if i%2 == p%2 {
                i as u8                
            } else {
                0
            };
            assert_eq!(vga.raw_read_mem(p, i), expected);
        }
    }
}