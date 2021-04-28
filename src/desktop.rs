use super::*;
use raw_gl_context::*;

pub fn log(msg: &str)
{
    println!("{}", msg);
}

pub struct Stuff
{
    context: GlContext
}

impl Stuff
{
    pub fn new<T>(event_loop: &EventLoop<T>) -> (Window, Self, glow::Context, &'static str)
    {
        let window = WindowBuilder::new().build(&event_loop).unwrap();
        let config = GlConfig
        {
            version: (2, 0),
            profile: raw_gl_context::Profile::Compatibility,
            stencil_bits: 0,
            samples: Some(4),
            srgb: true,
            double_buffer: true,
            vsync: true,
            ..Default::default()
        };
        let context = GlContext::create(&window, config).unwrap();
        context.make_current();
        let gl = unsafe { glow::Context::from_loader_function(|symbol| context.get_proc_address(symbol) as *const _) };
        (window, Self { context }, gl, "#version 110")
    }

    pub fn resumed(&mut self, _window: &Window) {}

    pub fn active(&self) -> bool { true }

    pub fn suspended(&mut self) {}

    pub fn swap_buffers(&self)
    {
        self.context.swap_buffers();
    }
}

pub type Instant = std::time::Instant;
pub type Duration = std::time::Duration;
pub fn instant() -> Instant { Instant::now() }
pub fn secs(duration: Duration) -> f32 { duration.as_secs_f32() }
