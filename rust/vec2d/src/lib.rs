use std::ops::{Index, IndexMut};
use std::slice::SliceIndex;

pub struct Vec2D<T> {
    data: Vec<T>,
    width: usize,
    height: usize,
}

impl<T> Index<usize> for Vec2D<T> {
    type Output = [T];

    #[inline]
    fn index(&self, row: usize) -> &[T] {
        debug_assert!(row < self.height);

        let pos = row * self.width;

        &self.data[pos..pos + self.width]
    }
}

impl<T> IndexMut<usize> for Vec2D<T> {
    #[inline]
    fn index_mut(&mut self, row: usize) -> &mut [T] {
        debug_assert!(row < self.height);

        let pos = row * self.width;

        &mut self.data[pos..pos + self.width]
    }
}

impl<T: Copy> Vec2D<T> {
    pub fn new(width: usize, height: usize, value: T) -> Self {
        Self {
            data: vec![value; width * height],
            width,
            height,
        }
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.height
    }

    #[inline]
    pub fn get(&self, row: usize, column: usize) -> Option<&<usize as SliceIndex<[T]>>::Output> {
        self.data.get(row * self.width + column)
    }

    #[inline]
    pub fn get_mut(&mut self, row: usize, column: usize) -> Option<&mut <usize as SliceIndex<[T]>>::Output> {
        self.data.get_mut(row * self.width + column)
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, row: usize, column: usize) -> &<usize as SliceIndex<[T]>>::Output {
        self.data.get_unchecked(row * self.width + column)
    }

    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, row: usize, column: usize) -> &mut <usize as SliceIndex<[T]>>::Output {
        self.data.get_unchecked_mut(row * self.width + column)
    }

    #[inline]
    pub fn rows(&self) -> impl Iterator<Item = &[T]> {
        self.data.chunks(self.width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        let mut v: Vec2D<u8> = Vec2D::new(2, 3, 0xff);

        assert_eq!(v.width, 2);
        assert_eq!(v.height, 3);

        assert_eq!(v[0][0], 0xff);
        assert_eq!(v[2][1], 0xff);

        v[2][1] = 0;

        assert_eq!(v[2][0], 0xff);
        assert_eq!(v[2][1], 0);

        v.get_mut(2, 1).map(|v| *v = 1);
        assert_eq!(v[2][1], 1);

        assert_eq!(v.get_mut(2, 2), None);
    }
}
