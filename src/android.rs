use super::*;
use khronos_egl as egl;

pub fn log(msg: &str)
{
    println!("{}", msg);
}

pub(crate) struct Stuff
{
    instance: egl::Instance<egl::Dynamic<libloading::Library, egl::EGL1_1>>,
    display: egl::Display,
    config: egl::Config,
    context: egl::Context,
    surface: Option<egl::Surface>
}

impl Stuff
{
    pub(crate) fn new<T>(event_loop: &EventLoop<T>) -> (Window, Self, glow::Context, &'static str,  &'static str)
    {
        //window
        let window = WindowBuilder::new().build(&event_loop).unwrap();
        //instance
        let lib = unsafe { libloading::Library::new("libEGL.so") }.unwrap();
        let instance = unsafe { egl::DynamicInstance::<egl::EGL1_1>::load_required_from(lib) }.unwrap();
        //display
        let display = instance.get_display(egl::DEFAULT_DISPLAY).unwrap();
        instance.initialize(display).unwrap();
        //context
        let attributes =
        [
            egl::RED_SIZE, 8,
            egl::GREEN_SIZE, 8,
            egl::BLUE_SIZE, 8,
            egl::ALPHA_SIZE, 8,
            egl::STENCIL_SIZE, 0,
            egl::NONE
        ];
        let config = instance.choose_first_config(display, &attributes).unwrap().unwrap();
        let context_attributes =
        [
            egl::CONTEXT_MAJOR_VERSION, 2,
            egl::CONTEXT_MINOR_VERSION, 0,
            egl::NONE
        ];
        let context = instance.create_context(display, config, None, &context_attributes).unwrap();
        //gl
        let gl = unsafe
        {
            glow::Context::from_loader_function(|symbol|
            {
                match instance.get_proc_address(symbol)
                {
                    Some(addr) => addr as *const _,
                    None => std::ptr::null()
                }
            })
        };
        (window, Self { instance, display, config, context, surface: None }, gl, "#version 100\nprecision mediump float;", "#version 100\nprecision mediump float;")
    }

    pub(crate) fn init(&mut self, window: &Window)
    {
        if self.surface.is_none()
        {
            use raw_window_handle::*;
            let handle = match window.raw_window_handle()
            {
                RawWindowHandle::Android(handle) => handle,
                _ => unreachable!()
            };
            let surface_attributes =
            [
                //egl::GL_COLORSPACE, egl::GL_COLORSPACE_SRGB,
                egl::NONE
            ];
            self.surface = Some(unsafe { self.instance.create_window_surface(self.display, self.config, handle.a_native_window, Some(&surface_attributes)) }.unwrap());
            self.instance.make_current(self.display, self.surface, self.surface, Some(self.context)).unwrap();
            self.instance.swap_interval(self.display, 1).unwrap();
        }
    }

    pub(crate) fn active(&self) -> bool
    {
        self.surface.is_some()
    }

    pub(crate) fn deinit(&mut self)
    {
        if self.surface.is_some()
        {
            self.instance.destroy_surface(self.display, self.surface.take().unwrap()).unwrap();
        }
    }

    pub(crate) fn swap_buffers(&self)
    {
        self.instance.swap_buffers(self.display, *self.surface.as_ref().unwrap()).unwrap();
    }
}

impl Drop for Stuff
{
    fn drop(&mut self)
    {
        self.instance.make_current(self.display, None, None, None).unwrap();
        self.instance.destroy_context(self.display, self.context).unwrap();
        self.instance.terminate(self.display).unwrap();
    }
}

pub mod time
{
    #[derive(Clone, Copy)]
    pub struct Instant(std::time::Instant);
    pub fn now() -> Instant { Instant(std::time::Instant::now()) }
    pub fn duration_secs(first: Instant, second: Instant) -> f32 { (second.0 - first.0).as_secs_f32() }
}

#[cfg(feature = "fs")]
pub mod fs
{
    use std::io::{BufReader, BufWriter, prelude::*};
    use std::ffi::CString;
    use std::sync::mpsc::{channel, Receiver};
    use std::io::Read;

    pub(crate) struct File
    {
        receiver: Receiver<(String, Vec<u8>)>,
        data: Option<(String, Vec<u8>)>
    }

    impl File
    {
        pub(crate) fn load(name: &str) -> Self
        {
            let c_name = CString::new(name).unwrap();
            let name = name.to_string();
            let (sender, receiver) = channel();
            std::thread::spawn(move ||
            {
                let mut contents = Vec::new();
                ndk_glue::native_activity().asset_manager().open(&c_name).unwrap().read_to_end(&mut contents).unwrap();
                sender.send((name, contents)).unwrap();
            });
            Self { receiver, data: None } 
        }

        pub(crate) fn finished(&mut self) -> bool
        {
            if self.data.is_some() { return true; }
            match self.receiver.try_recv()
            {
                Ok(data) => { self.data = Some(data); true },
                Err(_) => false
            }
        }

        pub(crate) fn get(self) -> Option<(String, Vec<u8>)>
        {
            self.data
        }
    }

    pub struct Storage
    {
        path: String
    }

    impl Storage
    {
        pub(crate) fn load() -> Self
        {
            let path = ndk_glue::native_activity().internal_data_path().to_str().unwrap().to_string();
            Self { path }
        }

        pub fn set(&mut self, key: &str, value: Option<&str>)
        {
            if let Some(value) = value { BufWriter::new(std::fs::File::create(&format!("{}/{}", self.path, key)).unwrap()).write_all(value.as_bytes()).unwrap(); }
            else { std::fs::remove_file(&format!("{}/{}", self.path, key)).ok(); }
        }

        pub fn get(&self, key: &str) -> Option<String>
        {
            std::fs::File::open(&format!("{}/{}", self.path, key)).ok().map(|file|
            {
                let mut contents = Vec::new();
                BufReader::new(file).read_to_end(&mut contents).unwrap();
                String::from_utf8(contents).unwrap()
            })
        }
    }
}
