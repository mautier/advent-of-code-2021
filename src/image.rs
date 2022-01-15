/// A simple 2D image.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Image<T> {
    pub height: u16,
    pub width: u16,
    /// Linear buffer of pixels, in row-major order.
    pub data: Vec<T>,
}

impl<T> Image<T> {
    pub fn len(&self) -> usize {
        self.height as usize * self.width as usize
    }

    pub fn size_hw(&self) -> (u16, u16) {
        (self.height, self.width)
    }

    pub fn pixel(&self, row: u16, col: u16) -> &T {
        &self.data[row as usize * self.width as usize + col as usize]
    }

    pub fn pixel_mut(&mut self, row: u16, col: u16) -> &mut T {
        &mut self.data[row as usize * self.width as usize + col as usize]
    }

    /// Enumerates the pixels, yielding (row, col, &pixel) tuples.
    pub fn enumerate_pixels(&self) -> impl Iterator<Item = (u16, u16, &T)> {
        self.data.iter().enumerate().map(|(lin_idx, px)| {
            (
                (lin_idx / self.width as usize) as u16,
                (lin_idx % self.width as usize) as u16,
                px,
            )
        })
    }

    /// Enumerates the pixels, yielding (row, col, &mut pixel) tuples.
    pub fn enumerate_pixels_mut(&mut self) -> impl Iterator<Item = (u16, u16, &mut T)> {
        self.data.iter_mut().enumerate().map(|(lin_idx, px)| {
            (
                (lin_idx / self.width as usize) as u16,
                (lin_idx % self.width as usize) as u16,
                px,
            )
        })
    }
}

impl<T: Clone> Image<T> {
    pub fn new_with_same_shape(other: &Self, fill_value: T) -> Self {
        Self {
            height: other.height,
            width: other.width,
            data: vec![fill_value; other.len()],
        }
    }
}
