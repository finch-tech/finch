use byteorder::{LittleEndian, WriteBytesExt};

pub struct VarInt(u64);

impl From<usize> for VarInt {
    fn from(i: usize) -> Self {
        VarInt(i as u64)
    }
}

impl VarInt {
    pub fn serialize(&self, stream: &mut Vec<u8>) {
        match self.0 {
            0...0xfc => {
                stream.write_u8(self.0 as u8).unwrap();
            }
            0xfd...0xffff => {
                stream.push(0xfdu8);
                stream.write_u16::<LittleEndian>(self.0 as u16).unwrap();
            }
            0x10000...0xffff_ffff => {
                stream.push(0xfeu8);
                stream.write_u32::<LittleEndian>(self.0 as u32).unwrap();
            }
            _ => {
                stream.push(0xffu8);
                stream.write_u64::<LittleEndian>(self.0 as u64).unwrap();
            }
        }
    }
}