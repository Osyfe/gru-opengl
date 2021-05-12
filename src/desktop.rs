use super::*;
use raw_gl_context::*;

pub fn log(msg: &str)
{
    println!("{}", msg);
}

pub(crate) struct Stuff
{
    context: GlContext
}

impl Stuff
{
    pub(crate) fn new<T>(event_loop: &EventLoop<T>) -> (Window, Self, glow::Context, fs::Storage, &'static str,  &'static str)
    {
        let mut builder = WindowBuilder::new();
        #[cfg(target_os = "windows")]
        {
            use winit::platform::windows::WindowBuilderExtWindows;
            builder = builder.with_drag_and_drop(false); //conflicts with rodio
        }
        let window = builder.build(&event_loop).unwrap();
        let config = GlConfig
        {
            version: (2, 0),
            profile: raw_gl_context::Profile::Compatibility,
            stencil_bits: 0,
            samples: None,
            srgb: true,
            double_buffer: true,
            vsync: true,
            ..Default::default()
        };
        let context = GlContext::create(&window, config).unwrap();
        context.make_current();
        let gl = unsafe { glow::Context::from_loader_function(|symbol| context.get_proc_address(symbol) as *const _) };
        (window, Self { context }, gl, fs::Storage::load(), "#version 110", "#version 110")
    }

    pub(crate) fn init(&mut self, _window: &Window) {}

    pub(crate) fn active(&self) -> bool { true }

    pub(crate) fn deinit(&mut self) {}

    pub(crate) fn swap_buffers(&self)
    {
        self.context.swap_buffers();
    }
}

pub mod time
{
    #[derive(Clone, Copy)]
    pub struct Instant(std::time::Instant);
    pub fn now() -> Instant { Instant(std::time::Instant::now()) }
    pub fn duration_secs(first: Instant, second: Instant) -> f32 { (second.0 - first.0).as_secs_f32() }
}

pub mod fs
{
    use std::io::{BufReader, BufWriter, prelude::*};
    use std::sync::mpsc::{channel, Receiver};
    use std::collections::HashMap;

    pub(crate) struct File
    {
        receiver: Receiver<(String, Vec<u8>)>,
        data: Option<(String, Vec<u8>)>
    }

    impl File
    {
        pub(crate) fn load(name: &str) -> Self
        {
            let full_name = if cfg!(debug_assertions) { format!("export/data/{}", name) } else { format!("data/{}", name) };
            let name = name.to_string();
            let (sender, receiver) = channel();
            std::thread::spawn(move ||
            {
                let mut contents = Vec::new();
                BufReader::new(std::fs::File::open(&full_name).unwrap()).read_to_end(&mut contents).unwrap();
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
        map: HashMap<String, String>
    }

    impl Storage
    {
        const PATH: &'static str = if cfg!(debug_assertions) { "export/data/STORAGE.gru" } else { "data/STORAGE.gru" };

        pub(crate) fn load() -> Self
        {
            let map = match std::fs::File::open(Self::PATH)
            {
                Ok(file) =>
                {
                    let mut contents = Vec::new();
                    BufReader::new(file).read_to_end(&mut contents).unwrap();
                    bincode::deserialize(&contents).unwrap()
                },
                Err(_) => HashMap::new()
            };
            Self { map }
        }

        pub fn set(&mut self, key: &str, value: Option<&str>)
        {
            if let Some(value) = value { self.map.insert(key.to_string(), value.to_string()); }
            else { self.map.remove(key); }
        }

        pub fn get(&self, key: &str) -> Option<String>
        {
            self.map.get(key).map(|value| value.to_string())
        }
    }

    impl Drop for Storage
    {
        fn drop(&mut self)
        {
            let contents: Vec<u8> = bincode::serialize(&self.map).unwrap();
            BufWriter::new(std::fs::File::create(Self::PATH).unwrap()).write_all(&contents).unwrap();
        }
    }
}
