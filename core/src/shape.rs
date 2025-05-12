/// Struct for storing the information about the shape of the image. This struct is being used with
/// the construction of image inside the speciesnet crates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Square(u32),
    Rectangular(u32, u32),
}

impl Shape {
    /// Retrieves the width of the stored shape.
    pub fn width(&self) -> u32 {
        match *self {
            Self::Square(len) => len,
            Self::Rectangular(width, _height) => width,
        }
    }

    /// Retrieves the height of the stored shape.
    pub fn height(&self) -> u32 {
        match *self {
            Self::Square(len) => len,
            Self::Rectangular(_width, height) => height,
        }
    }
}
