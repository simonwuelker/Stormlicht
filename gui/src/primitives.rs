#[derive(Clone, Copy, Debug)]
pub struct Point {
    x: i32,
    y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn y(&self) -> i32 {
        self.y
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn with_x(mut self, x: i32) -> Self {
        self.x = x;
        self
    }

    pub fn with_y(mut self, y: i32) -> Self {
        self.y = y;
        self
    }

    pub fn with_width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    pub fn with_height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    pub fn contains(&self, point: Point) -> bool {
        self.x <= point.x
            && point.x < self.x + self.width as i32
            && self.y <= point.y
            && point.y < self.y + self.height as i32
    }
}

// SDL2 trait impls
// Rect
impl From<Rect> for sdl2::rect::Rect {
    fn from(value: Rect) -> Self {
        sdl2::rect::Rect::new(value.x, value.y, value.width, value.height)
    }
}

impl From<sdl2::rect::Rect> for Rect {
    fn from(value: sdl2::rect::Rect) -> Self {
        Self {
            x: value.x(),
            y: value.y(),
            width: value.width(),
            height: value.height(),
        }
    }
}
