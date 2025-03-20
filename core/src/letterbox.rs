use image::Rgb;

use crate::shape::Shape;

#[derive(Debug, Clone, Copy)]
pub struct LetterboxOptionsBuilder {
    shape: Shape,
    scale_up: bool,
    auto: bool,
    stride: u32,
    scale_fill: bool,
    color: Rgb<u8>,
}

impl Default for LetterboxOptionsBuilder {
    fn default() -> Self {
        Self {
            shape: Shape::Square(640),
            scale_up: true,
            auto: true,
            stride: 32,
            scale_fill: false,
            color: Rgb([114, 114, 114]),
        }
    }
}

impl LetterboxOptionsBuilder {
    pub fn shape(&mut self, shape: Shape) -> &mut Self {
        self.shape = shape;
        self
    }

    pub fn scale_up(&mut self, scale_up: bool) -> &mut Self {
        self.scale_up = scale_up;
        self
    }

    pub fn auto(&mut self, auto: bool) -> &mut Self {
        self.auto = auto;
        self
    }

    pub fn stride(&mut self, stride: u32) -> &mut Self {
        self.stride = stride;
        self
    }

    pub fn scale_fill(&mut self, scale_fill: bool) -> &mut Self {
        self.scale_fill = scale_fill;
        self
    }

    pub fn color(&mut self, color: Rgb<u8>) -> &mut Self {
        self.color = color;
        self
    }

    pub fn build(&self) -> LetterboxOptions {
        LetterboxOptions {
            shape: self.shape,
            scale_up: self.scale_up,
            auto: self.auto,
            stride: self.stride,
            scale_fill: self.scale_fill,
            color: self.color,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LetterboxOptions {
    shape: Shape,
    scale_up: bool,
    auto: bool,
    stride: u32,
    scale_fill: bool,
    color: Rgb<u8>,
}

impl LetterboxOptions {
    pub fn builder() -> LetterboxOptionsBuilder {
        LetterboxOptionsBuilder::default()
    }

    pub fn shape(&self) -> Shape {
        self.shape
    }

    pub fn scale_up(&self) -> bool {
        self.scale_up
    }

    pub fn auto(&self) -> bool {
        self.auto
    }

    pub fn stride(&self) -> u32 {
        self.stride
    }

    pub fn scale_fill(&self) -> bool {
        self.scale_fill
    }

    pub fn color(&self) -> Rgb<u8> {
        self.color
    }
}
