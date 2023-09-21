use math::Vec2D;

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Mouse(MouseEvent),
}

#[derive(Clone, Copy, Debug)]
pub struct MouseEvent {
    pub position: Vec2D<i32>,
    pub kind: MouseEventKind,
}

#[derive(Clone, Copy, Debug)]
pub enum MouseEventKind {
    Down(MouseButton),
    Move,
    Up(MouseButton),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}
