use core::fmt;

#[derive(PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct Tag(pub u32); // Array of four uint8s (length = 32 bits) used to identify a table, design-variation axis, script, language system, feature, or baseline

impl Tag {
    #[inline]
    pub const fn from_be_bytes(bytes: [u8; 4]) -> Self {
        Self(u32::from_be_bytes(bytes))
    }
    #[inline]
    pub fn to_array(&self) -> [u8; 4] {
        self.0.to_be_bytes()
    }
    #[inline]
    pub fn to_string(&self) -> String {
        self.to_array().iter().map(|&c| c as char).collect()
    }
}

impl fmt::Debug for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{0} (= 0x{0:08x} = {1})", self.0, self.to_string())
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

pub const NAME: Tag = Tag::from_be_bytes(*b"name");

pub type TableTag = Tag;
pub type Offset32 = u32;
pub type Offset16 = u16;
