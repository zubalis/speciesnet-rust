/// Struct for storing the information about the shape of the image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Square(u32),
    Rectangular(u32, u32),
}

impl Shape {
    pub fn width(&self) -> u32 {
        match *self {
            Self::Square(len) => len,
            Self::Rectangular(width, _height) => width,
        }
    }

    pub fn height(&self) -> u32 {
        match *self {
            Self::Square(len) => len,
            Self::Rectangular(_width, height) => height,
        }
    }
}
