use winit::
{
    event::{Event as RawEvent, WindowEvent},
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

mod gl;
pub use gl::*;

pub fn start<T: App>()
{
    let event_loop = EventLoop::new();
    let (window, mut stuff, gl, glsl_vertex_header, glsl_fragment_header) = Stuff::new(&event_loop);
    let mut gl = Gl::new(gl, glsl_vertex_header, glsl_fragment_header);
    
    #[cfg(not(target_os = "android"))]
    let mut app = T::init(gl.init());

    #[cfg(target_os = "android")]
    let mut app = None;

    let mut then = instant();
    event_loop.run(move |event, _, control_flow|
    {
        match event
        {
            RawEvent::WindowEvent { event: WindowEvent::CloseRequested, .. } =>
            {
                *control_flow = ControlFlow::Exit
            },
            RawEvent::WindowEvent { event: WindowEvent::MouseInput { .. }, .. } =>
            {
                *control_flow = ControlFlow::Exit
            },
            RawEvent::WindowEvent { event: WindowEvent::Touch(_), .. } =>
            {
                *control_flow = ControlFlow::Exit
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
                #[cfg(target_os = "android")]
                let app = app.as_mut().unwrap();

                let now = instant();
                let dt = secs(now - then);
                then = now;

                let dims = window.inner_size().into();
                gl.viewport(dims);
                app.frame(dt, &mut gl, dims.0 as f32 / dims.1 as f32);

                stuff.swap_buffers();
            },
            RawEvent::LoopDestroyed => {},
            _ => ()
        }
    });
}

#[derive(Clone, Copy)]
pub enum Event
{
    Bla
}

pub trait App: 'static
{
    fn init(gl: &mut Gl) -> Self;
    fn input(&mut self, event: Event);
    fn frame(&mut self, dt: f32, gl: &mut Gl, aspect: f32);
}
