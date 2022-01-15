use super::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C"
{
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(msg: &str);
}

pub(crate) struct Stuff;

impl Stuff
{
    pub(crate) fn new<T>(event_loop: &EventLoop<T>) -> (Window, Self, glow::Context, &'static str,  &'static str)
    {
        use winit::platform::web::WindowBuilderExtWebSys;
        use wasm_bindgen::JsCast;
        let web_window = web_sys::window().unwrap();
        let canvas: web_sys::HtmlCanvasElement = web_window.document().unwrap().get_element_by_id("canvas").unwrap().dyn_into().unwrap();
        let context: web_sys::WebGlRenderingContext = canvas.get_context("webgl").unwrap().unwrap().dyn_into().unwrap();
        let window = WindowBuilder::new().with_canvas(Some(canvas)).build(&event_loop).unwrap();
        let gl = glow::Context::from_webgl1_context(context);
        (window, Self, gl, "#version 100\nprecision mediump float;", "#version 100\nprecision mediump float;")
    }

    pub(crate) fn init(&mut self, _window: &Window) {}

    pub(crate) fn active(&self) -> bool { true }

    pub(crate) fn deinit(&mut self) {}

    pub(crate) fn swap_buffers(&self) {}
}

pub mod time
{
    #[derive(Clone, Copy)]
    pub struct Instant(f64);
    pub fn now() -> Instant { Instant(web_sys::window().unwrap().performance().unwrap().now()) }
    pub fn duration_secs(first: Instant, second: Instant) -> f32 { ((second.0 - first.0) / 1e3) as f32 }
}

#[cfg(feature = "fs")]
pub mod fs
{
    use web_sys::{XmlHttpRequest, XmlHttpRequestResponseType};
    use js_sys::Uint8Array;
    use crate::event::File as EventFile;

    pub(crate) struct File
    {
        name: String,
        key: u64,
        request: XmlHttpRequest,
        data: Option<Vec<u8>>
    }

    impl File
    {
        pub(crate) fn load(name: &str, key: u64) -> Self
        {
            let name = name.to_string();
            let request = XmlHttpRequest::new().unwrap();
            request.open_with_async("GET", &format!("data/{}", name), true).unwrap();
            request.set_response_type(XmlHttpRequestResponseType::Arraybuffer);
            request.send().unwrap();
            Self { name, key, request, data: None }
        }

        pub(crate) fn finished(&mut self) -> bool
        {
            if self.data.is_some() { return true; }
            if self.request.ready_state() == 4 && self.request.status().unwrap() == 200 //DONE && OK
            {
                self.data = Some(Uint8Array::new_with_byte_offset(&self.request.response().unwrap(), 0).to_vec());
                true
            } else { false }
        }

        pub(crate) fn get(self) -> Option<EventFile>
        {
            let path = self.name;
            let key = self.key;
            self.data.map(|data| EventFile { path, key, data })
        }
    }

    pub struct Storage
    {
        pub(crate) storage: web_sys::Storage
    }

    impl Storage
    {
        pub(crate) fn load() -> Self
        {
            Self { storage: web_sys::window().unwrap().local_storage().unwrap().unwrap() }
        }

        pub fn set(&mut self, key: &str, value: Option<&str>)
        {
            if let Some(value) = value { self.storage.set_item(key, value).unwrap(); }
            else { self.storage.remove_item(key).unwrap(); }
        }

        pub fn get(&self, key: &str) -> Option<String>
        {
            self.storage.get_item(key).unwrap()
        }
    }
}
