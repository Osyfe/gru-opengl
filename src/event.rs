pub use winit::event::{VirtualKeyCode as KeyCode, MouseButton, TouchPhase};

pub enum Scroll
{
    Wheel(f32, f32),
    Touch(f32, f32)
}

pub struct File
{
    pub path: String,
    pub key: u64,
    pub data: Vec<u8>
}

pub enum Event
{
    Key { key: KeyCode, pressed: bool },
    Char(char),
    Click { button: MouseButton, pressed: bool },
    Cursor { position: (f32, f32) },
    Scroll(Scroll),
    Touch { position: (f32, f32), phase: TouchPhase, finger: u64 },
    #[cfg(feature = "fs")]
    File(Result<File, String>)
}
