use winit::{dpi::PhysicalSize, event::{ElementState, Event as RawEvent, KeyboardInput, MouseScrollDelta, WindowEvent, DeviceEvent}, event_loop::{ControlFlow, EventLoop}, window::{Window, WindowBuilder, Fullscreen, Icon}};

pub const DEBUG: bool = cfg!(debug_assertions);

#[cfg(not(target_arch = "wasm32"))]
mod desktop;
#[cfg(not(target_arch = "wasm32"))]
pub use desktop::*;

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use web::*;

pub mod event;
use event::*;
pub mod gl;

#[cfg(feature = "resource")]
pub mod resource;
#[cfg(feature = "ui_legacy")]
pub mod ui_legacy;
#[cfg(feature = "ui")]
pub mod ui;

#[cfg(feature = "rodio")]
use rodio::{OutputStream, OutputStreamHandle};

trait StuffTrait: Sized
{
    fn new<T>(event_loop: &EventLoop<T>) -> (Window, Self, glow::Context, &'static str,  &'static str);
    fn swap_buffers(&self);
}

#[cfg(feature = "loading")]
trait FileTrait: Sized
{
    fn load(name: &str, key: u64) -> Self;
    fn finished(&mut self) -> bool;
    fn get(self) -> Option<Result<File, String>>;
}

#[cfg(feature = "storage")]
trait StorageTrait: Sized
{
    fn load() -> Self;
    fn set(&mut self, key: &str, value: Option<&str>);
    fn get(&self, key: &str) -> Option<String>;
}

pub fn start<T: App>(init: T::Init)
{
    let event_loop = EventLoop::new();
    let (window, stuff, gl, glsl_vertex_header, glsl_fragment_header) = Stuff::new(&event_loop);
    let gl = gl::Gl::new(gl, glsl_vertex_header, glsl_fragment_header);
    let window_dims = window.inner_size().into();
    let mut ctx = Context
    {
        window,
        window_dims,
        gl,
		#[cfg(feature = "loading")]
        files: Vec::new(),
        #[cfg(feature = "storage")]
        storage: storage::Storage::load(),
        #[cfg(feature = "rodio")]
        audio_device: None
    };
    let mut app = T::init(&mut ctx, init);
    let mut then = time::now();

    event_loop.run(move |event, _, control_flow|
    {
        match event
        {
            RawEvent::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } =>
            {
                app.input(&mut ctx, Event::RawMouse { delta: (delta.0 as f32, delta.1 as f32) });
            },
            RawEvent::WindowEvent { event: WindowEvent::Resized(PhysicalSize { width, height }), .. } =>
            {
                ctx.window_dims = (width, height)
            },
            RawEvent::WindowEvent { event: WindowEvent::CloseRequested, .. } =>
            {
                *control_flow = ControlFlow::Exit
            },
            RawEvent::WindowEvent { event: WindowEvent::KeyboardInput { input: KeyboardInput { state, virtual_keycode: Some(key), .. }, .. }, .. } =>
            {
                app.input(&mut ctx, Event::Key { key, pressed: state == ElementState::Pressed });
            },
            RawEvent::WindowEvent { event: WindowEvent::ReceivedCharacter(ch), .. } =>
            {
               app.input(&mut ctx, Event::Char(ch));
            },
            RawEvent::WindowEvent { event: WindowEvent::MouseInput { button, state, .. }, .. } =>
            {
                #[cfg(feature = "rodio")]
                if ctx.audio_device.is_none()
                {
                    ctx.audio_device = Some(OutputStream::try_default().unwrap());
                }
                app.input(&mut ctx, Event::Click { button, pressed: state == ElementState::Pressed });
            },
            RawEvent::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } =>
            {
                let position: (f32, f32) = position.into();
                let w_dim_1 = ctx.window_dims.1 as f32;
                app.input(&mut ctx, Event::Cursor { position: (position.0, w_dim_1 - position.1) });
            },
            RawEvent::WindowEvent { event: WindowEvent::CursorLeft { .. }, .. } =>
            {
                app.input(&mut ctx, Event::CursorGone);
            },
            RawEvent::WindowEvent { event: WindowEvent::MouseWheel { delta, .. }, .. } =>
            {
                app.input(&mut ctx, Event::Scroll(match delta
                {
                    MouseScrollDelta::LineDelta(x, y) => Scroll::Wheel(x, y),
                    MouseScrollDelta::PixelDelta(p) => Scroll::Touch(p.x as f32, p.y as f32)
                }));
            },
            RawEvent::MainEventsCleared =>
            {
                #[cfg(feature = "loading")]
                for file in ctx.check_files().into_iter()
                {
                    app.input(&mut ctx, Event::File(file));
                }

                let now = time::now();
                let dt = time::duration_secs(then, now);
                then = now;

                ctx.gl.window_dims = ctx.window_dims;
                if !app.frame(&mut ctx, dt) { *control_flow = ControlFlow::Exit; }

                stuff.swap_buffers();
            },
            RawEvent::LoopDestroyed =>
            {
                app.deinit(&mut ctx);
            },
            _ => ()
        }
    });
}

pub struct Context
{
    window: Window,
    window_dims: (u32, u32),
    gl: gl::Gl,
	#[cfg(feature = "loading")]
    files: Vec<loading::File>,
    #[cfg(feature = "storage")]
    storage: storage::Storage,
    #[cfg(feature = "rodio")]
    audio_device: Option<(OutputStream, OutputStreamHandle)>
}

impl Context
{
	#[inline]
	pub fn gl(&mut self) -> &mut gl::Gl
    {
        &mut self.gl
    }
	
    #[inline]
    pub fn set_title(&mut self, title: &str)
    {
        self.window.set_title(title);
    }

    #[inline]
    pub fn set_icon(&mut self, window_icon: Option<Icon>)
    {
        self.window.set_window_icon(window_icon);
    }

    #[inline]
    pub fn set_icon_raw(&mut self, colors: Vec<u8>, (width, height): (u32, u32))
    {
        let icon = Icon::from_rgba(colors, width, height);
        self.set_icon(icon.ok());
    }

    #[inline]
    pub fn window_dims(&self) -> (u32, u32)
    {
        self.window_dims
    }

    pub fn set_window_dims(&mut self, (width, height): (u32, u32))
    {
        self.window.set_inner_size(PhysicalSize { width, height });
    }

    #[inline]
    pub fn fullscreen(&self) -> bool
    {
        self.window.fullscreen().is_some()
    }

    pub fn set_fullscreen(&mut self, open: bool)
    {
        let fullscreen = if open { Some(Fullscreen::Borderless(None)) } else { None };
        self.window.set_fullscreen(fullscreen);
    }

    pub fn mouse_cam_mode(&mut self, enable: bool)
    {
        if enable
        {
            self.window.set_cursor_visible(false);
            self.window.set_cursor_grab(true).unwrap();
        } else
        {
            self.window.set_cursor_grab(false).unwrap();
            self.window.set_cursor_visible(true);
        }
    }
}

#[cfg(feature = "loading")]
impl Context
{
    pub fn load_file(&mut self, name: &str, key: u64)
    {
        self.files.push(loading::File::load(name, key));
    }

    fn check_files(&mut self) -> Vec<Result<File, String>>
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

#[cfg(feature = "storage")]
impl Context
{
	pub fn set_storage(&mut self, key: &str, value: Option<&str>)
    {
        self.storage.set(key, value);
    }

    pub fn get_storage(&self, key: &str) -> Option<String>
    {
        self.storage.get(key)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_storage_keys(&self) -> Vec<String>
    {
        self.storage.keys()
    }
}

#[cfg(feature = "rodio")]
impl Context
{
    pub fn audio(&self) -> Option<&OutputStreamHandle>
    {
        self.audio_device.as_ref().map(|(_, device)| device)
    }
}

pub trait App: 'static
{
    type Init: 'static;
    fn init(ctx: &mut Context, init: Self::Init) -> Self;
    fn input(&mut self, ctx: &mut Context, event: Event);
    fn frame(&mut self, ctx: &mut Context, dt: f32) -> bool;
    fn deinit(&mut self, ctx: &mut Context);
}
