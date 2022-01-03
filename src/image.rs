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

    pub fn pixel(&self, row: u16, col: u16) -> &T {
        &self.data[row as usize * self.width as usize + col as usize]
    }

    pub fn pixel_mut(&mut self, row: u16, col: u16) -> &mut T {
        &mut self.data[row as usize * self.width as usize + col as usize]
    }
}
