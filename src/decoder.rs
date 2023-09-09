use std::mem::size_of;

use crate::data_types::{Fixed, Tag, Version16Dot16, F2DOT14, LONGDATETIME};

pub trait FromData: Sized {
    const SIZE: usize;
    fn parse(data: &[u8]) -> Option<Self>;
}

impl FromData for u8 {
    const SIZE: usize = size_of::<Self>();
    fn parse(data: &[u8]) -> Option<Self> {
        data.try_into().map(Self::from_be_bytes).ok()
    }
}

impl FromData for u16 {
    const SIZE: usize = size_of::<Self>();
    fn parse(data: &[u8]) -> Option<Self> {
        data.try_into().map(Self::from_be_bytes).ok()
    }
}

impl FromData for u32 {
    const SIZE: usize = size_of::<Self>();
    fn parse(data: &[u8]) -> Option<Self> {
        data.try_into().map(Self::from_be_bytes).ok()
    }
}

impl FromData for i8 {
    const SIZE: usize = size_of::<Self>();
    fn parse(data: &[u8]) -> Option<Self> {
        data.try_into().map(Self::from_be_bytes).ok()
    }
}

impl FromData for i16 {
    const SIZE: usize = size_of::<Self>();
    fn parse(data: &[u8]) -> Option<Self> {
        data.try_into().map(Self::from_be_bytes).ok()
    }
}

impl FromData for i32 {
    const SIZE: usize = size_of::<Self>();
    fn parse(data: &[u8]) -> Option<Self> {
        data.try_into().map(Self::from_be_bytes).ok()
    }
}

impl FromData for i64 {
    const SIZE: usize = size_of::<Self>();
    fn parse(data: &[u8]) -> Option<Self> {
        data.try_into().map(Self::from_be_bytes).ok()
    }
}

impl FromData for f32 {
    const SIZE: usize = size_of::<Self>();
    fn parse(data: &[u8]) -> Option<Self> {
        data.try_into().map(Self::from_be_bytes).ok()
    }
}

impl FromData for Tag {
    const SIZE: usize = u32::SIZE;
    fn parse(data: &[u8]) -> Option<Self> {
        type T = u32;
        T::parse(data).map(Self)
    }
}

impl FromData for Fixed {
    const SIZE: usize = i32::SIZE;
    fn parse(data: &[u8]) -> Option<Self> {
        type T = i32;
        T::parse(data).map(Self)
    }
}

impl FromData for LONGDATETIME {
    const SIZE: usize = i64::SIZE;
    fn parse(data: &[u8]) -> Option<Self> {
        type T = i64;
        T::parse(data).map(Self)
    }
}

impl FromData for Version16Dot16 {
    const SIZE: usize = u32::SIZE;
    fn parse(data: &[u8]) -> Option<Self> {
        type T = u32;
        T::parse(data).map(Self)
    }
}

impl FromData for F2DOT14 {
    const SIZE: usize = i16::SIZE;
    fn parse(data: &[u8]) -> Option<Self> {
        type T = i16;
        T::parse(data).map(Self)
    }
}

#[derive(Clone)]
pub struct Stream<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Stream<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    pub fn read_bytes(&mut self, len: usize) -> Option<&'a [u8]> {
        let bytes = self.data.get(self.offset..self.offset + len);
        self.offset += len;
        bytes
    }

    pub fn read<T: FromData>(&mut self) -> Option<T> {
        let data = self.read_bytes(T::SIZE)?;
        // read_bytes でバイト長が保証されているので， parse は成功する．
        Some(T::parse(data).unwrap())
    }

    pub fn read_array<T: FromData>(&mut self, count: usize) -> Option<Vec<T>> {
        let len = count * T::SIZE;
        let data = self.read_bytes(len)?;
        let mut s = Stream::new(data);
        let mut v = vec![];
        for _ in 0..count {
            // read_bytes() が成功しているので，バイト長は保証され， unwrap できる．
            v.push(s.read().unwrap());
        }
        Some(v)
    }

    pub fn read_unsized_array<T>(
        &mut self,
        data_count: usize,
        data_size: usize,
        parse: Box<dyn Fn(&'a [u8]) -> Option<T>>,
    ) -> Option<Vec<T>> {
        let mut v = vec![];
        for _ in 0..data_count {
            // read_bytes() が成功しているので，バイト長は保証され， unwrap できる．
            let data = self.read_bytes(data_size)?;
            let value = parse(data).unwrap();
            v.push(value);
        }
        Some(v)
    }

    // 残りのバイト列をすべてVecにして返す．
    pub fn read_all_array<T: FromData>(&mut self) -> Option<Vec<T>> {
        let count = (self.data.len() - self.offset) / T::SIZE;
        self.read_array(count)
    }

    pub fn is_end(&self) -> bool {
        self.offset == self.data.len()
    }

    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }

    pub fn get_offset(&self) -> usize {
        self.offset
    }

    pub fn set_end(&mut self) {
        self.offset = self.data.len()
    }

    pub fn set_len(&mut self, len: usize) -> Option<()> {
        self.data = self.data.get(0..len)?;
        Some(())
    }

    pub fn get_tail(self) -> Option<&'a [u8]> {
        self.data.get(self.offset..self.data.len())
    }
}
