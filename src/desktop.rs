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

impl StuffTrait for Stuff
{
    fn new<T>(event_loop: &EventLoop<T>) -> (Window, Self, glow::Context, &'static str,  &'static str)
    {
        #[allow(unused_mut)]
        let mut builder = WindowBuilder::new();
        #[cfg(target_os = "windows")]
        {
            use winit::platform::windows::WindowBuilderExtWindows;
            builder = builder.with_drag_and_drop(false); //conflicts with cpal
        }
        let window = builder.build(&event_loop).unwrap();
        let config = GlConfig
        {
            version: (2, 0),
            profile: raw_gl_context::Profile::Compatibility,
            depth_bits: 24,
            stencil_bits: 0,
            samples: Some(4),
            srgb: true,
            double_buffer: true,
            vsync: true,
            ..Default::default()
        };
        let context = GlContext::create(&window, config).unwrap();
        context.make_current();
        let gl = unsafe { glow::Context::from_loader_function(|symbol| context.get_proc_address(symbol) as *const _) };
        (window, Self { context }, gl, "#version 110", "#version 110")
    }

    fn swap_buffers(&self)
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

#[cfg(feature = "fs")]
pub(crate) mod fs
{
    use std::io::{BufReader, BufWriter, prelude::*};
    use std::sync::mpsc::{channel, Receiver, TryRecvError};
    use ahash::AHashMap;
    use crate::event::File as EventFile;

    pub(crate) struct File
    {
        receiver: Receiver<Result<EventFile, String>>,
        data: Option<Result<EventFile, String>>
    }

    impl super::FileTrait for File
    {
        fn load(name: &str, key: u64) -> Self
        {
            let full_name = if cfg!(debug_assertions) { format!("export/data/{}", name) } else { format!("data/{}", name) };
            let name = name.to_string();
            let (sender, receiver) = channel();
            std::thread::spawn(move ||
            {
                let mut contents = Vec::new();
                let file = match std::fs::File::open(&full_name)
                {
                    Ok(file) => file,
                    Err(err) =>
                    {
                        sender.send(Err(format!("{:?}", err))).unwrap();
                        return;
                    }
                };
                match BufReader::new(file).read_to_end(&mut contents)
                {
                    Ok(_) => {},
                    Err(err) =>
                    {
                        sender.send(Err(format!("{:?}", err))).unwrap();
                        return;
                    }
                }
                sender.send(Ok(EventFile { path: name, key, data: contents })).unwrap();
            });
            Self { receiver, data: None } 
        }

        fn finished(&mut self) -> bool
        {
            if self.data.is_some() { return true; }
            match self.receiver.try_recv()
            {
                Ok(data) => { self.data = Some(data); true },
                Err(TryRecvError::Disconnected) => { self.data = Some(Err("Loading Thread Disconnected".to_string())); true },
                Err(TryRecvError::Empty) => false
            }
        }

        fn get(self) -> Option<Result<EventFile, String>>
        {
            self.data
        }
    }

    pub(crate) struct Storage
    {
        map: AHashMap<String, String>
    }

    impl Storage
    {
        const PATH: &'static str = if super::DEBUG { "export/data/STORAGE.gru" } else { "data/STORAGE.gru" };

        pub fn keys(&self) -> Vec<String> 
        {
            self.map.keys().cloned().collect()
        }
    }

    impl super::StorageTrait for Storage
    {
        fn load() -> Self
        {
            let map = match std::fs::File::open(Self::PATH)
            {
                Ok(file) =>
                {
                    let mut contents = Vec::new();
                    BufReader::new(file).read_to_end(&mut contents).unwrap();
                    bincode::deserialize(&contents).unwrap()
                },
                Err(_) => AHashMap::new()
            };
            Self { map }
        }

        fn set(&mut self, key: &str, value: Option<&str>)
        {
            if let Some(value) = value { self.map.insert(key.to_string(), value.to_string()); }
            else { self.map.remove(key); }
        }

        fn get(&self, key: &str) -> Option<String>
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
