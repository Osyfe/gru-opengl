use winit::
{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder}
};
use glow::HasContext;

#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
mod desktop
{
    use super::*;

    pub fn init<T>(event_loop: &EventLoop<T>) -> (Window, raw_gl_context::GlContext, glow::Context, &'static str)
    {
        let window = WindowBuilder::new().build(&event_loop).unwrap();
        use raw_gl_context::{GlConfig, GlContext};
        let config = GlConfig
        {
            version: (2, 0),
            profile: raw_gl_context::Profile::Core,
            samples: Some(4),
            srgb: true,
            double_buffer: true,
            vsync: true,
            ..Default::default()
        };
        let context = GlContext::create(&window, config).unwrap();
        context.make_current();
        let gl = unsafe { glow::Context::from_loader_function(|symbol| context.get_proc_address(symbol) as *const _) };
        (window, context, gl, "#version 110")
    }
}

#[cfg(target_arch = "wasm32")]
mod web
{
    use super::*;

    pub fn init<T>(event_loop: &EventLoop<T>) -> (Window, glow::Context, &'static str)
    {
        use winit::platform::web::WindowBuilderExtWebSys;
        use wasm_bindgen::JsCast;
        let canvas: web_sys::HtmlCanvasElement = web_sys::window().unwrap().document().unwrap().get_element_by_id("canvas").unwrap().dyn_into().unwrap();
        let context: web_sys::WebGlRenderingContext = canvas.get_context("webgl").unwrap().unwrap().dyn_into().unwrap();
        let gl = glow::Context::from_webgl1_context(context);
        let window = WindowBuilder::new().with_canvas(Some(canvas)).build(&event_loop).unwrap();
        (window, gl, "#version 100 es")
    }
}

#[cfg(target_os = "android")]
mod android
{
    use super::*;
    use khronos_egl as egl;

    /*
    pub fn log(msg: &str)
    {
        let rust = std::ffi::CString::new("Rust").unwrap();
        let msg = std::ffi::CString::new(msg).unwrap();
        ndk_glue::android_log(log::Level::Error, &rust, &msg);
    }
    */

    pub struct Stuff
    {
        pub instance: egl::Instance<egl::Dynamic<libloading::Library, egl::EGL1_1>>,
        pub display: egl::Display,
        pub config: egl::Config,
        pub context: egl::Context,
        pub surface: Option<egl::Surface>
    }

    impl Stuff
    {
        pub fn new<T>(event_loop: &EventLoop<T>) -> (Window, Self, glow::Context, &'static str)
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

            (window, Self { instance, display, config, context, surface: None }, gl, "#version 100 es")
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
                    khronos_egl::NONE
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
}

pub fn start()
{
    let event_loop = EventLoop::new();

    #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
    let (window, context, gl, glsl_version) = desktop::init(&event_loop);

    #[cfg(target_arch = "wasm32")]
    let (window, gl, glsl_version) = web::init(&event_loop);

    #[cfg(target_os = "android")]
    let (window, mut stuff, gl, version) = android::Stuff::new(&event_loop);
    
    let mut t = 0.0;
    event_loop.run(move |event, _, control_flow|
    {
        match event
        {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } =>
            {
                *control_flow = ControlFlow::Exit
            },
            Event::WindowEvent { event: WindowEvent::MouseInput { .. }, .. } =>
            {
                *control_flow = ControlFlow::Exit
            },
            Event::WindowEvent { event: WindowEvent::Touch(_), .. } =>
            {
                *control_flow = ControlFlow::Exit
            },
            Event::Resumed =>
            {
                #[cfg(target_os = "android")]
                stuff.resumed(&window);
            },
            Event::Suspended =>
            {
                #[cfg(target_os = "android")]
                stuff.suspended();
            },
            Event::MainEventsCleared => unsafe
            {
                #[cfg(target_os = "android")]
                if !stuff.active() { return; }

                t += 0.01;

                gl.clear_color(1.0, t % 1.0, 0.0, 1.0);
                gl.clear(glow::COLOR_BUFFER_BIT);

                #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
                context.swap_buffers();

                #[cfg(target_os = "android")]
                stuff.swap_buffers();
            },
            Event::LoopDestroyed =>
            {
            }
            _ => ()
        }
    });
}
