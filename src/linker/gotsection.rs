#![allow(unused)]
use super::common::*;
use super::symbol::Symbol;
use super::output::Chunk;
#[derive(Default, Debug)]
pub struct GotSection {
    pub Chunk:      Chunk,
    pub GotTpSyms:  Vec<Rc<RefCell<Symbol>>>
}

#[derive(Default, Clone)]
#[repr(C)]
pub struct GotEntry {
    pub Idx:    usize,
    pub Val:    u64
}

impl Deref for GotSection {
    type Target = Chunk;
    fn deref(&self) -> &Self::Target {
        &self.Chunk
    }
}

impl DerefMut for GotSection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.Chunk
    }
}

impl GotSection {
    pub fn new() -> Box<Self> {
        let mut g = Self {
            Chunk: Chunk::new(),
            GotTpSyms: vec![],
        };

        g.Name = ".got".into();
        g.Shdr.Type = abi::SHT_PROGBITS;
        g.Shdr.Flags = (abi::SHF_ALLOC | abi::SHF_WRITE) as u64;
        g.Shdr.AddrAlign = 0;

        g.into()
    }

    pub fn AddGotTpSymbol(&mut self, sym: Rc<RefCell<Symbol>>) {
        sym.borrow_mut().GotTpIdx = self.Shdr.Size / 8;
        self.Shdr.Size += 8;
        self.GotTpSyms.push(sym.clone());
    }

    pub fn GetEntries(&self, ctx: *mut Box<Context>) -> Vec<GotEntry> {
        let mut entries = vec![];

        for sym in &self.GotTpSyms {
            let idx = sym.borrow().GotTpIdx;
            entries.push(
                GotEntry { Idx: idx, Val: sym.borrow().GetAddr() - ptr2ref(ctx).TpAddr }
            );
        }

        entries
    }
}

/* generate immediate value for instructions */

pub fn itype(val: u32) -> u32 {
    val << 20
}

pub fn stype(val: u32) -> u32 {
    (Bits(val, 11, 5) << 25) |
    (Bits(val, 4, 0) << 7)
}

pub fn btype(val: u32) -> u32 {
    (Bit(val, 12) << 31) | 
    (Bits(val, 10, 5) << 25) |
    (Bits(val, 4, 1) << 8) | 
    (Bit(val, 11) << 7)
}

pub fn utype(val: u32) -> u32 {
    val.wrapping_add(0x800) & 0xffff_f000
}

pub fn jtype(val: u32) -> u32 {
    (Bit(val, 20) << 31) |
    (Bits(val, 10, 1) << 21) |
    (Bit(val, 11) << 20) | 
    (Bits(val, 19, 12) << 12)
}

pub fn cbtype(val: u16) -> u16 {
    (Bit(val, 8) << 12) |
    (Bit(val, 4) << 11) |
    (Bit(val, 3) << 10) |
    (Bit(val, 7) << 6) |
    (Bit(val, 6) << 5) |
    (Bit(val, 2) << 4) |
    (Bit(val, 1) << 3) |
    (Bit(val, 5) << 2)
}

pub fn cjtype(val: u16) -> u16 {
    (Bit(val, 11) << 12) |
    (Bit(val, 4) << 11) |
    (Bit(val, 9) << 10) |
    (Bit(val, 8) << 9) |
    (Bit(val, 10) << 8) |
    (Bit(val, 6) << 7) |
    (Bit(val, 7) << 6) |
    (Bit(val, 3) << 5) |
    (Bit(val, 2) << 4) |
    (Bit(val, 1) << 3) |
    (Bit(val, 5) << 2)
}

pub fn writeItype(loc: &mut [u8], val: u32) {
    let data = (Read::<u32>(loc) & 0b000000_00000_11111_111_11111_1111111) | stype(val);
    Write(loc, data);
}

pub fn writeStype(loc: &mut [u8], val: u32) {
    let data = (Read::<u32>(loc) & 0b000000_11111_11111_111_00000_1111111) | stype(val);
    Write(loc, data);
}

pub fn writeBtype(loc: &mut [u8], val: u32) {
    let data = (Read::<u32>(loc) & 0b000000_11111_11111_111_00000_1111111) | btype(val);
    Write(loc, data);
}

pub fn writeUtype(loc: &mut [u8], val: u32) {
    let data = (Read::<u32>(loc) & 0b000000_00000_00000_000_11111_1111111) | utype(val);
    Write(loc, data);
}

pub fn writeJtype(loc: &mut [u8], val: u32) {
    let data = (Read::<u32>(loc) & 0b000000_00000_00000_000_11111_1111111) | jtype(val);
    Write(loc, data);
}

pub fn setRs1(loc: &mut [u8], rs1: u32) {
    let data = Read::<u32>(loc) & &0b111111_11111_00000_111_11111_1111111;
    Write(loc, data);
    let data = Read::<u32>(loc) | (rs1 << 15);
    Write(loc, data);
}