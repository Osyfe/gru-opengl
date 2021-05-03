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
use desktop::*;

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
use web::*;

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "android")]
use android::*;

pub mod event;
use event::*;
mod gl;
pub use gl::*;

pub fn start<T: App>()
{
    let event_loop = EventLoop::new();
    let (window, mut stuff, gl, glsl_vertex_header, glsl_fragment_header) = Stuff::new(&event_loop);
    let mut gl = Gl::new(gl, glsl_vertex_header, glsl_fragment_header);
    
    #[cfg(not(target_os = "android"))]
    let mut app = Some(T::init(gl.init()));

    #[cfg(target_os = "android")]
    let mut app: Option<T> = None;

    let mut dims = window.inner_size().into();
    let mut then = instant();

    event_loop.run(move |event, _, control_flow|
    {
        match event
        {
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
                if let Some(app) = &mut app { app.input(Event::Key { key, state }); }
            },
            RawEvent::WindowEvent { event: WindowEvent::MouseInput { button, state, .. }, .. } =>
            {
                if let Some(app) = &mut app { app.input(Event::Click { button, state }); }
            },
            RawEvent::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } =>
            {
                let position: (f32, f32) = position.into();
                if let Some(app) = &mut app { app.input(Event::Cursor { position: (position.0, dims.1 as f32 - position.1) }); }
            },
            RawEvent::WindowEvent { event: WindowEvent::MouseWheel { delta, .. }, .. } =>
            {
                if let Some(app) = &mut app { app.input(Event::Scroll(match delta
                {
                    MouseScrollDelta::LineDelta(x, y) => Scroll::Wheel(x, y),
                    MouseScrollDelta::PixelDelta(p) => Scroll::Touch(p.x as f32, p.y as f32)
                })); }
            },
            RawEvent::WindowEvent { event: WindowEvent::Touch(Touch { phase, location, id, .. }), .. } =>
            {
                let position: (f32, f32) = location.into();
                if let Some(app) = &mut app { app.input(Event::Touch { position: (position.0, dims.1 as f32 - position.1), phase, finger: id }); }
            },
            RawEvent::Resumed =>
            {
                stuff.resumed(&window);

                #[cfg(target_os = "android")]
                if app.is_none() { app = Some(T::init(gl.init())); }
                
                then = instant();
            },
            RawEvent::Suspended =>
            {
                stuff.suspended();
            },
            RawEvent::MainEventsCleared => if stuff.active()
            {
                let app = app.as_mut().unwrap();

                let now = instant();
                let dt = secs(now - then);
                then = now;

                gl.window_dims = dims;
                if !app.frame(dt, &mut gl, dims) { *control_flow = ControlFlow::Exit; }

                stuff.swap_buffers();
            },
            RawEvent::LoopDestroyed => {},
            _ => ()
        }
    });
}

pub trait App: 'static
{
    fn init(gl: &mut Gl) -> Self;
    fn input(&mut self, event: Event);
    fn frame(&mut self, dt: f32, gl: &mut Gl, window_dims: (u32, u32)) -> bool;
}
