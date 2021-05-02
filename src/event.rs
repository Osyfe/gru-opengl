pub use winit::event::{ElementState, VirtualKeyCode as KeyCode, MouseButton, TouchPhase};

#[derive(Clone, Copy)]
pub enum Scroll
{
    Wheel(f32, f32),
    Touch(f32, f32)
}

#[derive(Clone, Copy)]
pub enum Event
{
    Key { key: KeyCode, state: ElementState },
    Click { button: MouseButton, state: ElementState },
    Cursor { position: (f32, f32) },
    Scroll(Scroll),
    Touch { position: (f32, f32), phase: TouchPhase, finger: u64 }
}
