use winit::{event::ElementState, keyboard::PhysicalKey};

#[derive(Debug, Clone)]
pub struct KeyboardInputEvent {
    pub key: PhysicalKey,
    pub state: ElementState,
    pub repeat: bool,
}

impl KeyboardInputEvent {
    pub fn new(key: PhysicalKey, state: ElementState, repeat: bool) -> Self {
        Self { key, state, repeat }
    }
}

#[derive(Debug, Clone)]
pub struct CursorMovedEvent {
    pub x: f64,
    pub y: f64,
}

impl CursorMovedEvent {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}
