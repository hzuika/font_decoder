use core::fmt;
use std::{cmp::Ordering, marker::PhantomData};

use crate::data_types::{Fixed, Tag, Version16Dot16, LONGDATETIME};

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

impl FromData for i16 {
    const SIZE: usize = 2;
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

impl FromData for i64 {
    const SIZE: usize = 64;
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

impl FromData for LONGDATETIME {
    const SIZE: usize = 8;
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

// 実行時に要素のサイズが確定するデータの配列
pub struct UnsizedLazyArray<'a, T> {
    buffer: &'a [u8],
    data_size: usize,
    parse_data: Box<dyn Fn(&'a [u8]) -> Option<T>>,
    data_type: PhantomData<T>,
}

impl<'a, T> UnsizedLazyArray<'a, T> {
    pub fn new(
        buffer: &'a [u8],
        data_size: usize,
        parse_data: Box<dyn Fn(&'a [u8]) -> Option<T>>,
    ) -> UnsizedLazyArray<'a, T> {
        UnsizedLazyArray::<'a, T> {
            buffer,
            data_size,
            parse_data,
            data_type: PhantomData::<T>,
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len() / self.data_size
    }

    pub fn get(&self, index: usize) -> Option<T> {
        if index < self.len() {
            let start = index * self.data_size;
            let end = start + self.data_size;
            self.buffer
                .get(start..end)
                .and_then(|x| (self.parse_data)(x))
        } else {
            None
        }
    }
}

// スライスのラッパーなので，ファットポインタ分のサイズしか持たない．
#[derive(Clone, Copy)]
pub struct LazyArray<'a, T> {
    buffer: &'a [u8],
    data_type: PhantomData<T>,
}

impl<'a, T: FromData> LazyArray<'a, T> {
    pub fn new(data: &'a [u8]) -> LazyArray<'a, T> {
        LazyArray {
            buffer: data,
            data_type: PhantomData::<T>,
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len() / T::SIZE
    }

    pub fn get(&self, index: usize) -> Option<T> {
        if index < self.len() {
            let start = index * T::SIZE;
            let end = start + T::SIZE;
            self.buffer.get(start..end).and_then(T::parse)
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn last(&self) -> Option<T> {
        if !self.is_empty() {
            self.get(self.len() - 1)
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

pub struct UnsizedLazyArrayIter<'a, 'b, T> {
    array: &'a UnsizedLazyArray<'b, T>,
    index: usize,
}

pub struct LazyArrayIter<'a, T> {
    array: &'a LazyArray<'a, T>,
    index: usize,
}

impl<'a, 'b, T> Iterator for UnsizedLazyArrayIter<'a, 'b, T> {
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

impl<'a, 'b, T> IntoIterator for &'a UnsizedLazyArray<'b, T> {
    type IntoIter = UnsizedLazyArrayIter<'a, 'b, T>;
    type Item = T;
    fn into_iter(self) -> Self::IntoIter {
        UnsizedLazyArrayIter {
            array: self,
            index: 0,
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

impl<'a, T: FromData + fmt::Debug> fmt::Debug for LazyArray<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.into_iter()).finish()
    }
}

impl<'a, T: fmt::Debug> fmt::Debug for UnsizedLazyArray<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.into_iter()).finish()
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

    pub fn read_unsized_array<T>(
        &mut self,
        data_count: usize,
        data_size: usize,
        parse_data: Box<dyn Fn(&'a [u8]) -> Option<T>>,
    ) -> Option<UnsizedLazyArray<'a, T>> {
        let len = data_count * data_size;
        self.read_bytes(len)
            .map(|data| UnsizedLazyArray::new(data, data_size, parse_data))
    }

    pub fn at_end(&self) -> bool {
        self.offset == self.data.len()
    }

    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }

    pub fn get_offset(&self) -> usize {
        self.offset
    }

    pub fn set_len(&mut self, len: usize) -> Option<()> {
        self.data = self.data.get(0..len)?;
        Some(())
    }

    pub fn tail(self) -> Option<&'a [u8]> {
        self.data.get(self.offset..self.data.len())
    }
}

#[cfg(test)]
mod tests {
    use super::{FromData, LazyArray, LazyArrayIter, UnsizedLazyArray, UnsizedLazyArrayIter};

    #[test]
    fn test_lazy_array() {
        let a = [1_u8, 2, 3, 4];
        let b = LazyArray::<u16>::new(&a);
        let mut c = b.into_iter();
        assert_eq!(c.next(), Some(0x0102));
        assert_eq!(c.next(), Some(0x0304));
        assert_eq!(c.next(), None);
    }

    #[test]
    fn test_lazy_array_iter() {
        let a = [1_u8, 2, 3, 4];
        let b = LazyArray::<u16>::new(&a);
        let mut c = LazyArrayIter {
            array: &b,
            index: 0,
        };
        assert_eq!(c.next(), Some(0x0102));
        assert_eq!(c.next(), Some(0x0304));
        assert_eq!(c.next(), None);
    }

    #[test]
    fn test_unsized_lazy_array_iter() {
        let a = [1_u8, 2, 3, 4];
        let b = UnsizedLazyArray::<u16>::new(&a, 2, Box::new(u16::parse));
        let mut c = UnsizedLazyArrayIter {
            array: &b,
            index: 0,
        };
        assert_eq!(c.next(), Some(0x0102));
        assert_eq!(c.next(), Some(0x0304));
        assert_eq!(c.next(), None);
    }

    #[test]
    fn test_struct_with_closure() {
        struct A {
            _f: Box<dyn Fn()>,
        }
        struct B<'a> {
            _a: &'a A,
        }

        let a = A {
            _f: Box::new(|| {}),
        };
        let _b = B { _a: &a };

        struct C<'a> {
            _f: Box<dyn Fn(&'a [u8])>,
        }
        // need 2 lifetime annotation.
        struct D<'a, 'b> {
            _c: &'a C<'b>,
        }
        let c = C {
            _f: Box::new(|_: &[u8]| {}),
        };
        let _d = D { _c: &c };

        struct E;
        struct F<'a> {
            _e: &'a E,
        }
        struct G<'a> {
            _f: &'a F<'a>,
        }
        let e = E;
        let f = F { _e: &e };
        let _g = G { _f: &f };
    }
}
