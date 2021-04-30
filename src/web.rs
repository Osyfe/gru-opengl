use super::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C"
{
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(msg: &str);
}

pub struct Stuff;

impl Stuff
{
    pub fn new<T>(event_loop: &EventLoop<T>) -> (Window, Self, glow::Context, &'static str,  &'static str)
    {
        use winit::platform::web::WindowBuilderExtWebSys;
        use wasm_bindgen::JsCast;
        let canvas: web_sys::HtmlCanvasElement = web_sys::window().unwrap().document().unwrap().get_element_by_id("canvas").unwrap().dyn_into().unwrap();
        let context: web_sys::WebGlRenderingContext = canvas.get_context("webgl").unwrap().unwrap().dyn_into().unwrap();
        let gl = glow::Context::from_webgl1_context(context);
        let window = WindowBuilder::new().with_canvas(Some(canvas)).build(&event_loop).unwrap();
        (window, Self, gl, "#version 100\nprecision mediump float;", "#version 100\nprecision mediump float;")
    }

    pub fn resumed(&mut self, _window: &Window) {}

    pub fn active(&self) -> bool { true }

    pub fn suspended(&mut self) {}

    pub fn swap_buffers(&self) {}
}

pub type Instant = f64;
pub type Duration = f64;
pub fn instant() -> Instant { web_sys::window().unwrap().performance().unwrap().now() }
pub fn secs(duration: Duration) -> f32 { (duration / 1e3) as f32 }
