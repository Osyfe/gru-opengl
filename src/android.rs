use super::*;
use khronos_egl as egl;

pub fn log(msg: &str)
{
    let rust = std::ffi::CString::new("Rust").unwrap();
    let msg = std::ffi::CString::new(msg).unwrap();
    ndk_glue::android_log(log::Level::Error, &rust, &msg);
}

pub struct Stuff
{
    instance: egl::Instance<egl::Dynamic<libloading::Library, egl::EGL1_1>>,
    display: egl::Display,
    config: egl::Config,
    context: egl::Context,
    surface: Option<egl::Surface>
}

impl Stuff
{
    pub fn new<T>(event_loop: &EventLoop<T>) -> (Window, Self, glow::Context, &'static str,  &'static str)
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
        let gl = unsafe { glow::Context::from_loader_function(|symbol|
        {
            match instance.get_proc_address(symbol)
            {
                Some(addr) => addr as *const _,
                None => std::ptr::null()
            }
        }) };

        (window, Self { instance, display, config, context, surface: None }, gl, "#version 100\nprecision mediump float;", "#version 100\nprecision mediump float;")
    }

    pub fn resumed(&mut self, window: &Window)
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

    pub fn active(&self) -> bool
    {
        self.surface.is_some()
    }

    pub fn suspended(&mut self)
    {
        if self.surface.is_some()
        {
            self.instance.destroy_surface(self.display, self.surface.take().unwrap()).unwrap();
        }
    }

    pub fn swap_buffers(&self)
    {
        self.instance.swap_buffers(self.display, *self.surface.as_ref().unwrap()).unwrap();
    }
}

impl Drop for Stuff
{
    fn drop(&mut self)
    {
        self.instance.make_current(self.display, None, None, None).unwrap();
        self.suspended();
        self.instance.destroy_context(self.display, self.context).unwrap();
        self.instance.terminate(self.display).unwrap();
    }
}

pub type Instant = std::time::Instant;
pub type Duration = std::time::Duration;
pub fn instant() -> Instant { Instant::now() }
pub fn secs(duration: Duration) -> f32 { duration.as_secs_f32() }
