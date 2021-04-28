use winit::
{
    event::{Event as RawEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder}
};
use glow::HasContext;

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

pub fn start<T: App>()
{
    let event_loop = EventLoop::new();
    let (window, mut stuff, gl, _glsl_header) = Stuff::new(&event_loop);
    let mut app = T::init(&Gl);
    
    let mut t = 0.0;
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
                then = instant();
            },
            RawEvent::Suspended =>
            {
                stuff.suspended();
            },
            RawEvent::MainEventsCleared => unsafe
            {
                if !stuff.active() { return; }

                let now = instant();
                let dt = secs(now - then);
                then = now;

                app.frame(dt, &Gl);
                t += 0.5 * dt;

                gl.enable(glow::FRAMEBUFFER_SRGB);
                gl.clear_color(1.0, t % 1.0, 0.0, 1.0);
                gl.clear(glow::COLOR_BUFFER_BIT);

                stuff.swap_buffers();
            },
            RawEvent::LoopDestroyed => { }
            _ => ()
        }
    });
}

#[derive(Clone, Copy)]
pub enum Event
{
    Bla
}

pub struct Gl;

pub trait App: 'static
{
    fn init(gl: &Gl) -> Self;
    fn input(&mut self, event: Event);
    fn frame(&mut self, dt: f32, gl: &Gl);
}
