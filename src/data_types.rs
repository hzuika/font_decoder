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
pub const FVAR: Tag = Tag::from_be_bytes(*b"fvar");
pub const STAT: Tag = Tag::from_be_bytes(*b"STAT");
pub const CMAP: Tag = Tag::from_be_bytes(*b"cmap");
pub const OS_2: Tag = Tag::from_be_bytes(*b"OS/2");
pub const LOCA: Tag = Tag::from_be_bytes(*b"loca");
pub const HEAD: Tag = Tag::from_be_bytes(*b"head");
pub const MAXP: Tag = Tag::from_be_bytes(*b"maxp");
// 32-bit signed fixed-point number (16.16)
#[derive(PartialEq)]
pub struct Fixed(pub i32);

impl Fixed {
    pub fn to_f64(&self) -> f64 {
        // assert_eq!(65536, 1 << 16)
        f64::from(self.0) / 65536.0
    }
}

impl fmt::Debug for Fixed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{0} (= 0x{0:x}, {1})", self.0, self.to_f64())
    }
}

#[derive(Debug)]
pub struct LONGDATETIME(pub i64); // Date and time represented in number of seconds since 12:00 midnight, January 1, 1904, UTC. The value is represented as a signed 64-bit integer.

// Ex: version 0.5 is 0x00005000
#[derive(Debug)]
pub struct Version16Dot16(pub u32); // Packed 32-bit value with major and minor version numbers.

pub type TableTag = Tag;
pub type Offset32 = u32;
pub type Offset16 = u16;
#[allow(non_camel_case_types)]
pub type uint32 = u32;
#[allow(non_camel_case_types)]
pub type uint16 = u16;
#[allow(non_camel_case_types)]
pub type int16 = i16;
#[allow(non_camel_case_types)]
pub type uint8 = u8;
