use winit::
{
    dpi::PhysicalSize,
    event::{Event as RawEvent, WindowEvent, KeyboardInput, MouseScrollDelta, Touch},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder}
};

#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
mod desktop;
#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
pub use desktop::*;

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use web::*;

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "android")]
pub use android::*;

pub mod event;
use event::*;
pub mod gl;

pub fn start<T: App>()
{
    let event_loop = EventLoop::new();
    let (window, mut stuff, gl, storage, glsl_vertex_header, glsl_fragment_header) = Stuff::new(&event_loop);
    let gl = gl::Gl::new(gl, glsl_vertex_header, glsl_fragment_header);
    let mut ctx = Context { gl, storage, files: Vec::new() };
    let mut app = None;

    let mut dims = window.inner_size().into();
    let mut then = time::now();

    event_loop.run(move |event, _, control_flow|
    {
        match event
        {
            #[cfg(not(target_os = "android"))]
            RawEvent::NewEvents(winit::event::StartCause::Init) =>
            {
                stuff.init(&window);
                ctx.gl.init();
                app = Some(T::init(&mut ctx));
            },
            #[cfg(target_os = "android")]
            RawEvent::Resumed =>
            {
                stuff.init(&window);
                if app.is_none()
                {
                    ctx.gl.init();
                    app = Some(T::init(&mut ctx));
                }
                then = time::now();
            },
            RawEvent::LoopDestroyed | RawEvent::Suspended =>
            {
                stuff.deinit();
                if let Some(app) = app.take() { app.deinit(&mut ctx); }
            },
            RawEvent::WindowEvent { event: WindowEvent::Resized(PhysicalSize { width, height }), .. } =>
            {
                dims = (width, height)
            }
            RawEvent::WindowEvent { event: WindowEvent::CloseRequested, .. } =>
            {
                *control_flow = ControlFlow::Exit
            },
            RawEvent::WindowEvent { event: WindowEvent::KeyboardInput { input: KeyboardInput { state, virtual_keycode: Some(key), .. }, .. }, .. } =>
            {
                if let Some(app) = &mut app { app.input(&mut ctx, Event::Key { key, state }); }
            },
            RawEvent::WindowEvent { event: WindowEvent::MouseInput { button, state, .. }, .. } =>
            {
                if let Some(app) = &mut app { app.input(&mut ctx, Event::Click { button, state }); }
            },
            RawEvent::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } =>
            {
                let position: (f32, f32) = position.into();
                if let Some(app) = &mut app { app.input(&mut ctx, Event::Cursor { position: (position.0, dims.1 as f32 - position.1) }); }
            },
            RawEvent::WindowEvent { event: WindowEvent::MouseWheel { delta, .. }, .. } =>
            {
                if let Some(app) = &mut app { app.input(&mut ctx, Event::Scroll(match delta
                {
                    MouseScrollDelta::LineDelta(x, y) => Scroll::Wheel(x, y),
                    MouseScrollDelta::PixelDelta(p) => Scroll::Touch(p.x as f32, p.y as f32)
                })); }
            },
            RawEvent::WindowEvent { event: WindowEvent::Touch(Touch { phase, location, id, .. }), .. } =>
            {
                let position: (f32, f32) = location.into();
                if let Some(app) = &mut app { app.input(&mut ctx, Event::Touch { position: (position.0, dims.1 as f32 - position.1), phase, finger: id }); }
            },
            RawEvent::MainEventsCleared => if stuff.active()
            {
                let app = app.as_mut().unwrap();

                for (name, data) in ctx.check_files().into_iter()
                {
                    app.input(&mut ctx, Event::File(name, data));
                }

                let now = time::now();
                let dt = time::duration_secs(then, now);
                then = now;

                ctx.gl.window_dims = dims;
                if !app.frame(&mut ctx, dt, dims) { *control_flow = ControlFlow::Exit; }

                stuff.swap_buffers();
            },
            _ => ()
        }
    });
}

pub struct Context
{
    pub gl: gl::Gl,
    pub storage: fs::Storage,
    files: Vec<fs::File>
}

impl Context
{
    pub fn load_file(&mut self, name: &str)
    {
        self.files.push(fs::File::load(name));
    }

    fn check_files(&mut self) -> Vec<(String, Vec<u8>)>
    {
        if self.files.len() == 0 { return Vec::new(); }
        let mut finished = Vec::new();
        let mut i = self.files.len();
        while i > 0
        {
            i -= 1;
            if self.files[i].finished()
            {
                let file = self.files.remove(i);
                finished.push(file.get().unwrap());
            }
        }
        finished
    }
}

pub trait App: 'static
{
    fn init(ctx: &mut Context) -> Self;
    fn input(&mut self, ctx: &mut Context, event: Event);
    fn frame(&mut self, ctx: &mut Context, dt: f32, window_dims: (u32, u32)) -> bool;
    fn deinit(self, ctx: &mut Context);
}
