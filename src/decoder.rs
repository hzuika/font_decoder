use std::{cmp::Ordering, marker::PhantomData};

use crate::data_types::{Fixed, Tag};

pub trait FromData: Sized {
    const SIZE: usize;
    fn parse(data: &[u8]) -> Option<Self>;
}

impl FromData for u8 {
    const SIZE: usize = 1;
    fn parse(data: &[u8]) -> Option<Self> {
        data.try_into().map(Self::from_be_bytes).ok()
    }
}

impl FromData for u16 {
    const SIZE: usize = 2;
    fn parse(data: &[u8]) -> Option<Self> {
        data.try_into().map(Self::from_be_bytes).ok()
    }
}

impl FromData for u32 {
    const SIZE: usize = 4;
    fn parse(data: &[u8]) -> Option<Self> {
        data.try_into().map(Self::from_be_bytes).ok()
    }
}

impl FromData for i32 {
    const SIZE: usize = 4;
    fn parse(data: &[u8]) -> Option<Self> {
        data.try_into().map(Self::from_be_bytes).ok()
    }
}

impl FromData for Tag {
    const SIZE: usize = 4;
    fn parse(data: &[u8]) -> Option<Self> {
        type T = u32;
        T::parse(data).map(Self)
    }
}

impl FromData for Fixed {
    const SIZE: usize = 4;
    fn parse(data: &[u8]) -> Option<Self> {
        type T = i32;
        T::parse(data).map(Self)
    }
}

pub struct LazyArray<'a, T> {
    data: &'a [u8],
    data_type: PhantomData<T>,
}

impl<'a, T: FromData> LazyArray<'a, T> {
    pub fn new(data: &'a [u8]) -> LazyArray<'a, T> {
        LazyArray {
            data,
            data_type: PhantomData::<T>,
        }
    }

    pub fn len(&self) -> usize {
        self.data.len() / T::SIZE
    }

    pub fn get(&self, index: usize) -> Option<T> {
        if index < self.len() {
            let start = index * T::SIZE;
            let end = start + T::SIZE;
            self.data.get(start..end).and_then(T::parse)
        } else {
            None
        }
    }

    pub fn binary_search_by<F: FnMut(&T) -> Ordering>(&self, mut f: F) -> Option<(usize, T)> {
        let mut size = self.len();
        if size == 0 {
            None
        } else {
            // 左端
            let mut base = 0;
            while size > 1 {
                // 検索範囲の長さ
                let half = size / 2;
                // 右端
                let mid = base + half;
                let cmp = f(&self.get(mid)?);
                base = if cmp == Ordering::Greater { base } else { mid };
                size -= half;
            }

            let value = self.get(base)?;
            if f(&value) == Ordering::Equal {
                Some((base, value))
            } else {
                None
            }
        }
    }
}

pub struct LazyArrayIter<'a, T> {
    array: &'a LazyArray<'a, T>,
    index: usize,
}

impl<'a, T: FromData> Iterator for LazyArrayIter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.array.len() {
            None
        } else {
            self.index += 1;
            self.array.get(self.index - 1)
        }
    }
}

impl<'a, T: FromData> IntoIterator for &'a LazyArray<'a, T> {
    type IntoIter = LazyArrayIter<'a, T>;
    type Item = T;
    fn into_iter(self) -> Self::IntoIter {
        LazyArrayIter {
            array: self,
            index: 0,
        }
    }
}

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
        self.read_bytes(T::SIZE).and_then(T::parse)
    }

    pub fn read_array<T: FromData>(&mut self, count: usize) -> Option<LazyArray<'a, T>> {
        let len = count * T::SIZE;
        self.read_bytes(len).map(LazyArray::new)
    }

    pub fn at_end(&self) -> bool {
        self.offset == self.data.len()
    }

    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }

    pub fn tail(self) -> Option<&'a [u8]> {
        self.data.get(self.offset..self.data.len())
    }
}
