use self::id::*;

use super::log;
use super::{event::File, gl::*, Context};
use ahash::AHashMap;
use std::fmt::Display;
use std::path::PathBuf;

pub mod load;

pub type ResL<'a> = &'a mut dyn ResLoad;
pub type ResIterMut<'a, 'b> = Box<dyn Iterator<Item = ResL<'b>> + 'a>;
pub const RESOURCE_SYSTEM_MAX_SIZE: u64 = 1000000;

pub fn get_res_iter_mut<'a, 'b, const N: usize>(arr: [ResL<'b>; N]) -> ResIterMut {
    Box::new(arr.into_iter())
}

pub struct ResSys<T: ResourceSystem> {
    res: T,
    id: u64,
    load_id: Id<u64>,
    loaded_counter: Id<u64>,
}

impl<T: ResourceSystem> ResSys<T> {
    fn load(&self) -> u64 {
        *self.load_id.current() - self.id * RESOURCE_SYSTEM_MAX_SIZE
    }

    fn loaded(&self) -> u64 {
        *self.loaded_counter.current()
    }

    pub fn get_resource_len(&mut self) -> usize {
        self.get_iter_mut().count()
    }

    pub fn finished_loading(&self) -> bool {
        //log(&format!("Loaded {}/{}", self.loaded(), self.load()));
        self.loaded() == self.load()
    }

    pub fn finished_loading_percentage(&self) -> f32 {
        self.loaded() as f32 / self.load() as f32
    }

    pub fn get_resource_status(&mut self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Current Loadstatus:\n").expect("Faild to write String");
        self.get_iter_mut().for_each(|res| {
            write!(f, "{}\n", res.display_string()).expect("Faild to write String");
        });
        write!(
            f,
            "Total Loaded: {:1}%",
            self.finished_loading_percentage() * 100.0
        )
    }

    pub fn add_file_event(&mut self, file: File, gl: &mut Gl) -> bool {
        self.add_file_event_wrapper(file, gl)
    }
}

impl<T: ResourceSystem> ResourceSystemWrapper for ResSys<T> {
    fn load_id(&mut self) -> &mut Id<u64> {
        &mut self.load_id
    }

    fn loaded_counter(&mut self) -> &mut Id<u64> {
        &mut self.loaded_counter
    }

    fn empty(id: u64) -> Self {
        Self {
            res: T::empty(),
            id,
            load_id: Id::new(id * RESOURCE_SYSTEM_MAX_SIZE),
            loaded_counter: Id::new(0),
        }
    }

    fn get_iter_mut(&mut self) -> ResIterMut {
        self.res.get_iter_mut()
    }

    fn load(&mut self, ctx: &mut Context) {
        self.res
            .get_iter_mut()
            .for_each(|res| res.load(&mut self.load_id, ctx));
    }
}

impl<T: ResourceSystem> std::ops::Deref for ResSys<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.res
    }
}

impl<T: ResourceSystem> std::ops::DerefMut for ResSys<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.res
    }
}

trait ResourceSystemWrapper: std::ops::Deref + Sized {
    fn load_id(&mut self) -> &mut Id<u64>;
    fn loaded_counter(&mut self) -> &mut Id<u64>;

    fn empty(id: u64) -> Self;
    fn get_iter_mut(&mut self) -> ResIterMut;

    fn load(&mut self, ctx: &mut Context);

    fn create_loading(id: u64, ctx: &mut Context) -> Self {
        let mut ret = Self::empty(id);
        ret.load(ctx);
        ret
    }

    fn add_file_event_wrapper(&mut self, file: File, gl: &mut Gl) -> bool {
        let res_load_option = Box::new(self.get_iter_mut()).find(|rl| rl.needs_key(&file.key));
        if let Some(res_load) = res_load_option {
            res_load.add_file(file, gl);
            self.loaded_counter().increase();
            true
        } else {
            false
        }
    }
}

pub trait ResourceSystem: Sized {
    fn empty() -> Self;
    fn get_iter_mut(&mut self) -> ResIterMut;
    fn new(id: u64) -> ResSys<Self> {
        ResSys::<Self>::empty(id)
    }

    fn new_loading(id: u64, ctx: &mut Context) -> ResSys<Self> {
        ResSys::<Self>::create_loading(id, ctx)
    }
}

#[macro_export]
macro_rules! impl_ResourceSystem {
    ($subj: ident = $(($name: ident, $dt: ty, $filename: expr, $config: expr)),+) => {
        pub struct $subj {
            $(pub $name: gru_opengl::resource::Res<$dt>), +
        }

        impl gru_opengl::resource::ResourceSystem for $subj {
            fn empty() -> Self {
                Self
                {
                    $($name: gru_opengl::resource::Res::new($filename, $config)), +
                }
            }

            fn get_iter_mut(&mut self) -> gru_opengl::resource::ResIterMut {
                Box::new([$(&mut self.$name as  gru_opengl::resource::ResL), +].into_iter())
            }
        }
    };
}

pub trait Load {
    type Config;
    fn load(key: &mut Id<u64>, path: &PathBuf, ctx: &mut Context) -> Loadprotocol;
    fn interpret(lp: &Loadprotocol, gl: &mut Gl, config: &mut Self::Config) -> Self;
    fn path(name: &'static str) -> PathBuf;
}
enum ResState<T> {
    Empty,
    Loading(Loadprotocol),
    Loaded(T),
}

impl<T: Load> ResState<T> {
    fn get(&self) -> Option<&T> {
        match self {
            ResState::Empty => None,
            ResState::Loading(_) => None,
            ResState::Loaded(res) => Some(res),
        }
    }
}

impl<T: Load> Display for ResState<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResState::Empty => write!(f, "Empty"),
            ResState::Loading(_) => write!(f, "Loading"),
            ResState::Loaded(_) => write!(f, "Loaded"),
        }
    }
}

pub struct Res<T: Load> {
    res: ResState<T>,
    path: PathBuf,
    config: T::Config,
}
pub trait ResLoad {
    fn load(&mut self, key_gen: &mut Id<u64>, ctx: &mut Context);
    fn interpret(&mut self, gl: &mut Gl);
    fn needs_key(&self, key: &u64) -> bool;
    fn add_file(&mut self, file: File, gl: &mut Gl);
    fn display_string(&self) -> String;
}

impl<T: 'static + Load> ResLoad for Res<T> {
    fn load(&mut self, key_gen: &mut Id<u64>, ctx: &mut Context) {
        log(&format!("Start loading {:?}", self.path));
        self.res = ResState::Loading(T::load(key_gen, &self.path, ctx));
    }

    fn interpret(&mut self, gl: &mut Gl) {
        if let ResState::Loading(lp) = &self.res {
            let name = &lp.name();
            self.res = ResState::Loaded(T::interpret(lp, gl, &mut self.config));
            log(&format!("Loaded {name}"));
        }
    }

    fn needs_key(&self, key: &u64) -> bool {
        self.needs_key(key)
    }

    fn add_file(&mut self, file: File, gl: &mut Gl) {
        self.add_file(file, gl);
    }

    fn display_string(&self) -> String {
        format!("{self}")
    }
}

impl<T: 'static + Load> Res<T> {
    pub fn get(&self) -> &T {
        self.res
            .get()
            .unwrap_or_else(|| panic!("Resource not loaded {:?}", self.path))
    }

    pub fn get_config(&self) -> &T::Config {
        &self.config
    }

    pub fn get_config_mut(&mut self) -> Option<&mut T::Config> {
        if let ResState::Loaded(_) = self.res {
            None
        } else {
            Some(&mut self.config)
        }
    }

    pub fn new(name: &'static str, config: T::Config) -> Self {
        Self {
            res: ResState::Empty,
            path: T::path(name),
            config,
        }
    }

    pub fn is_loaded(&self) -> bool {
        if let ResState::Loaded(_) = self.res {
            true
        } else {
            false
        }
    }

    fn needs_key(&self, key: &u64) -> bool {
        if let ResState::Loading(lp) = &self.res {
            lp.keys.contains_key(key)
        } else {
            false
        }
    }

    fn add_file(&mut self, file: File, gl: &mut Gl) {
        let mut complete = false;
        if let ResState::Loading(lp) = &mut self.res {
            lp.add_file(file);
            complete = lp.can_be_interpreted();
        }
        if complete {
            self.interpret(gl);
        }
    }
}

impl<T: 'static + Load> Display for Res<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {:?}", self.res, self.path)
    }
}
pub struct Loadprotocol {
    name: String,
    keys: AHashMap<u64, String>,
    files: AHashMap<String, File>,
    missing_files: usize,
}

impl Loadprotocol {
    fn add_file(&mut self, file: File) {
        let keyname = self.keys.get(&file.key).unwrap();
        self.files.insert(keyname.clone(), file);
        self.missing_files -= 1;
    }

    fn can_be_interpreted(&self) -> bool {
        self.missing_files == 0
    }

    pub fn get_data(&self, keyname: &str) -> &[u8] {
        &self.files.get(keyname).unwrap().data
    }

    pub fn empty(name: String) -> Self {
        let missing_files = 0;
        Loadprotocol {
            name,
            keys: AHashMap::new(),
            files: AHashMap::new(),
            missing_files,
        }
    }

    pub fn request_file(
        &mut self,
        key_gen: &mut Id<u64>,
        filename: &str,
        keyname: &str,
        ctx: &mut Context,
    ) {
        let key = key_gen.next();
        self.missing_files += 1;
        self.keys.insert(key, keyname.to_string());
        ctx.load_file(filename, key);
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}

pub mod id {
    #[derive(Debug, PartialEq, Clone)]
    pub struct Id<T: PartialEq + Copy + Clone + Increment> {
        value: T,
    }

    impl<T: PartialEq + Copy + Clone + Increment> Id<T> {
        pub fn new(start: T) -> Self {
            Id { value: start }
        }

        pub fn next(&mut self) -> T {
            let ret = self.value;
            self.value.increase();
            ret
        }

        pub fn current(&self) -> &T {
            &self.value
        }
    }

    pub trait Increment {
        fn increase(&mut self);
    }

    impl<T: PartialEq + Copy + Clone + Increment> Increment for Id<T> {
        fn increase(&mut self) {
            self.value.increase()
        }
    }

    impl Increment for u32 {
        fn increase(&mut self) {
            *self += 1;
        }
    }

    impl Increment for u64 {
        fn increase(&mut self) {
            *self += 1;
        }
    }
}
