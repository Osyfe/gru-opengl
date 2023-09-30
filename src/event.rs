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
    RawMouse { delta: (f32, f32) },
    Key { key: KeyCode, pressed: bool },
    Char(char),
    Click { button: MouseButton, pressed: bool },
    Cursor { position: (f32, f32) },
    CursorGone,
    Scroll(Scroll),
    #[cfg(feature = "loading")]
    File(Result<File, String>)
}
