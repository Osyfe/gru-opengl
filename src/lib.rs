use winit::
{dpi::PhysicalSize, event::{ElementState, Event as RawEvent, KeyboardInput, MouseScrollDelta, Touch, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::{Window, WindowBuilder, Fullscreen}};

pub const DEBUG: bool = cfg!(debug_assertions);

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
    let (window, mut stuff, gl, glsl_vertex_header, glsl_fragment_header) = Stuff::new(&event_loop);
    let gl = gl::Gl::new(gl, glsl_vertex_header, glsl_fragment_header);
    let window_dims = window.inner_size().into();
    #[cfg(feature = "fs")]
    let mut ctx = Context { window, window_dims, gl, storage: fs::Storage::load(), files: Vec::new() };
    #[cfg(not(feature = "fs"))]
    let mut ctx = Context { window, window_dims, gl };
    let mut app = None;

    let mut then = time::now();

    event_loop.run(move |event, _, control_flow|
    {
        match event
        {
            #[cfg(not(target_os = "android"))]
            RawEvent::NewEvents(winit::event::StartCause::Init) =>
            {
                stuff.init(&ctx.window);
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
                ctx.window_dims = (width, height)
            }
            RawEvent::WindowEvent { event: WindowEvent::CloseRequested, .. } =>
            {
                *control_flow = ControlFlow::Exit
            },
            RawEvent::WindowEvent { event: WindowEvent::KeyboardInput { input: KeyboardInput { state, virtual_keycode: Some(key), .. }, .. }, .. } =>
            {
                if let Some(app) = &mut app { app.input(&mut ctx, Event::Key { key, pressed: state == ElementState::Pressed }); }
            },
            RawEvent::WindowEvent { event: WindowEvent::MouseInput { button, state, .. }, .. } =>
            {
                if let Some(app) = &mut app { app.input(&mut ctx, Event::Click { button, pressed: state == ElementState::Pressed }); }
            },
            RawEvent::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } =>
            {
                let position: (f32, f32) = position.into();
                let w_dim_1 = ctx.window_dims.1 as f32;
                if let Some(app) = &mut app { app.input(&mut ctx, Event::Cursor { position: (position.0, w_dim_1 - position.1) }); }
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
                let w_dim_1 = ctx.window_dims.1 as f32;
                if let Some(app) = &mut app { app.input(&mut ctx, Event::Touch { position: (position.0, w_dim_1 - position.1), phase, finger: id }); }
            },
            RawEvent::MainEventsCleared => if stuff.active()
            {
                let app = app.as_mut().unwrap();

                #[cfg(feature = "fs")]
                for (name, data) in ctx.check_files().into_iter()
                {
                    app.input(&mut ctx, Event::File(name, data));
                }

                let now = time::now();
                let dt = time::duration_secs(then, now);
                then = now;

                ctx.gl.window_dims = ctx.window_dims;
                if !app.frame(&mut ctx, dt) { *control_flow = ControlFlow::Exit; }

                stuff.swap_buffers();
            },
            _ => ()
        }
    });
}

pub struct Context
{
    window: Window,
    window_dims: (u32, u32),
    pub gl: gl::Gl,
    #[cfg(feature = "fs")]
    pub storage: fs::Storage,
    #[cfg(feature = "fs")]
    files: Vec<fs::File>
}

#[cfg(feature = "fs")]
impl Context
{
    pub fn set_title(&mut self, title: &str)
    {
        self.window.set_title(title);
    }

    pub fn window_dims(&self) -> (u32, u32)
    {
        self.window_dims
    }

    pub fn set_window_dims(&mut self, (width, height): (u32, u32))
    {
        self.window.set_inner_size(PhysicalSize { width, height });
    }

    pub fn fullscreen(&self) -> bool
    {
        self.window.fullscreen().is_some()
    }

    pub fn set_fullscreen(&mut self, open: bool)
    {
        let fullscreen = if open { Some(Fullscreen::Borderless(None)) } else { None };
        self.window.set_fullscreen(fullscreen);
    }

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
    fn frame(&mut self, ctx: &mut Context, dt: f32) -> bool;
    fn deinit(self, ctx: &mut Context);
}
