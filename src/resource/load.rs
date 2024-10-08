use gltf::{accessor, buffer, Gltf, Semantic};
use image::GenericImageView;
use std::{path::PathBuf, time::Duration, sync::Arc};
use rodio::{Decoder, source::Source};

use super::*;
use std::collections::hash_map::Entry;

impl<T: AttributesReprCpacked> Load for Shader<T> {
    type Config = ();
    fn path(file_name: &'static str) -> PathBuf {
        PathBuf::from("shaders").join(file_name) //no extension because 2 files .vert .frag in function
    }

    fn load(key_gen: &mut Id<u64>, path: &PathBuf, ctx: &mut Context) -> Loadprotocol {
        let mut lp = Loadprotocol::empty(format!("Shader {path:?}"));
        lp.request_file(
            key_gen,
            &path.with_extension("vert").to_string_lossy(),
            "vert",
            ctx,
        );
        lp.request_file(
            key_gen,
            &path.with_extension("frag").to_string_lossy(),
            "frag",
            ctx,
        );
        lp
    }

    fn interpret(lp: &Loadprotocol, gl: &mut Gl, _: &mut Self::Config) -> Self {
        let vertex_glsl = String::from_utf8_lossy(lp.get_data("vert"));
        let fragment_glsl = String::from_utf8_lossy(lp.get_data("frag"));
        gl.new_shader(&vertex_glsl, &fragment_glsl)
    }
}

pub struct TextureLoadConfig {
    pub channel: crate::gl::TextureChannel,
    pub mipmap: bool,
    pub wrap_s: crate::gl::TextureWrap,
    pub wrap_t: crate::gl::TextureWrap,
}

impl TextureLoadConfig {
    fn add_size(&self, size: u32) -> TextureConfig {
        TextureConfig {
            size,
            channel: self.channel,
            mipmap: self.mipmap,
            wrap_s: self.wrap_s,
            wrap_t: self.wrap_t,
        }
    }
}

impl<const P: bool> Load for Texture<P> {
    type Config = TextureLoadConfig;
    fn path(file_name: &'static str) -> PathBuf {
        PathBuf::from("textures").join(file_name).with_extension("png")
    }

    fn load(key_gen: &mut Id<u64>, path: &PathBuf, ctx: &mut Context) -> Loadprotocol {
        let mut lp = Loadprotocol::empty(format!("Texture {path:?}"));
        lp.request_file(key_gen, &path.to_string_lossy(), "file", ctx);
        lp
    }

    fn interpret(lp: &Loadprotocol, gl: &mut Gl, config: &mut Self::Config) -> Self {
        let name = lp.name();
        let img = image::load_from_memory(lp.get_data("file")).unwrap();
        let (width, height) = img.dimensions();
        if width != height {
            panic!("Texture {name} is not quadratic (w/h) = ({width}/{height})")
        };
        let img = match config.channel {
            TextureChannel::RGBA => img.into_rgba8().into_raw(),
            TextureChannel::RGB => img.into_rgb8().into_raw(),
            TextureChannel::A => img.into_luma8().into_raw(),
        };
        gl.new_texture(&config.add_size(width), &img)
    }
}

pub struct VertexData {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 4],
    pub tex_coord: [f32; 2],
    pub color: [f32; 3],
}

pub trait BuildFromGltf: AttributesReprCpacked {
    fn build(data: VertexData) -> Self;
}

pub struct Model<V: BuildFromGltf> {
    pub vertices: VertexBuffer<V>,
    pub indices: IndexBuffer,
}

impl<V: BuildFromGltf> Load for Model<V> {
    type Config = ();
    fn path(file_name: &'static str) -> PathBuf {
        PathBuf::from("models").join(file_name)
    }

    fn load(key_gen: &mut Id<u64>, path: &PathBuf, ctx: &mut Context) -> Loadprotocol {
        let mut lp = Loadprotocol::empty(format!("Model {path:?}"));
        lp.request_file(
            key_gen,
            &path.with_extension("gltf").to_string_lossy(),
            "gltf",
            ctx,
        );
        lp.request_file(
            key_gen,
            &path.with_extension("bin").to_string_lossy(),
            "bin",
            ctx,
        );
        lp
    }

    fn interpret(lp: &Loadprotocol, gl: &mut Gl, _: &mut Self::Config) -> Self {
        let model_name = lp.name();
        let doc = Gltf::from_slice(lp.get_data("gltf")).unwrap().document;
        let mut bin = AHashMap::new();
        for buffer in doc.buffers() {
            match buffer.source() {
                buffer::Source::Uri(name) => {
                    if let Entry::Vacant(vac) = bin.entry(name) {
                        vac.insert(lp.get_data("bin"));
                    }
                }
                buffer::Source::Bin => unreachable!(),
            }
        }
        let mut meshes = doc.meshes();
        let mesh = meshes.next().unwrap();
        if meshes.next().is_some() {
            panic!("Model {model_name:?} has more than 1 mesh");
        }
        let name = mesh.name().unwrap().to_string();
        let mut indices = Vec::new();

        let mut primitives = mesh.primitives();
        let primitive = primitives.next().unwrap();
        if primitives.next().is_some() {
            panic!("Mesh {name} has more than 1 primitive");
        }

        let i0 = 0; //TODO dafuq means old code ->> = (positions.len() / 3) as u16;
        let accessor = primitive.indices().unwrap();
        let view = accessor.view().unwrap();
        let data = &(if let buffer::Source::Uri(name) = view.buffer().source() {
            bin.get(name).unwrap()
        } else {
            unreachable!()
        })[view.offset()..];
        let stride = view.stride().unwrap_or_else(|| accessor.size());
        for i in 0..accessor.count() {
            let start = (stride * i) + accessor.offset();
            let data = &data[start..(start + accessor.size())];
            match accessor.data_type() {
                accessor::DataType::U16 => {
                    for int in data.chunks_exact(2) {
                        indices.push(u16::from_ne_bytes(int.try_into().unwrap()) + i0);
                    }
                }
                accessor::DataType::U32 => {
                    for int in data.chunks_exact(4) {
                        indices.push(u32::from_ne_bytes(int.try_into().unwrap()) as u16 + i0);
                    }
                }
                _ => unreachable!(),
            }
        }

        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut tangents = Vec::new();
        let mut tex_coords = Vec::new();
        let mut colors = Vec::new();
        //let mut weights = Vec::new();
        //let mut joints = Vec::new();

        for attribute in primitive.attributes() {
            if let Some(vec) = match attribute.0 {
                Semantic::Positions => Some(&mut positions),
                Semantic::Normals => Some(&mut normals),
                Semantic::Tangents => Some(&mut tangents),
                Semantic::TexCoords(_) => Some(&mut tex_coords),
                Semantic::Colors(_) => Some(&mut colors),
                //Semantic::Weights(_) => Some(&mut weights),
                //Semantic::Joints(_) => Some(&mut joints),
                _ => None,
            } {
                let accessor = attribute.1;
                if accessor.data_type() != accessor::DataType::F32 {
                    panic!("ModelData::new: Model \"{model_name}\" contains not F32 data for {:?} (instead {:?}).", attribute.0, accessor.data_type());
                }
                let view = accessor.view().unwrap();
                let data = &(if let buffer::Source::Uri(name) = view.buffer().source() {
                    bin.get(name).unwrap()
                } else {
                    unreachable!()
                })[view.offset()..];
                let stride = view.stride().unwrap_or_else(|| accessor.size());
                for i in 0..accessor.count() {
                    let start = (stride * i) + accessor.offset();
                    let data = &data[start..(start + accessor.size())];
                    for float in data.chunks_exact(4) {
                        vec.push(f32::from_le_bytes(float.try_into().unwrap()));
                    }
                }
            }
        }

        let mut position_iter = positions.chunks_exact(3);
        let mut normal_iter = normals.chunks_exact(3);
        let mut tangent_iter = tangents.chunks_exact(4);
        let mut tex_coord_iter = tex_coords.chunks_exact(2);
        let mut color_iter = colors.chunks_exact(4);

        let mut vertices = Vec::new();
        for _ in 0..((positions.len() / 3)
            .max(normals.len() / 3)
            .max(tangents.len() / 4)
            .max(tex_coords.len() / 2)
            .max(colors.len() / 4))
        {
            let pos = position_iter.next().unwrap_or(&[0.0; 3]);
            let n = normal_iter.next().unwrap_or(&[0.0; 3]);
            let t = tangent_iter.next().unwrap_or(&[0.0; 4]);
            let tc = tex_coord_iter.next().unwrap_or(&[0.0; 2]);
            let c = color_iter.next().unwrap_or(&[0.0; 4]);

            vertices.push(V::build(VertexData {
                position: [pos[0], pos[1], pos[2]],
                normal: [n[0], n[1], n[2]],
                tangent: [t[0], t[1], t[2], t[3]],
                tex_coord: [tc[0], tc[1]],
                color: [c[0], c[1], c[2]],
            }));
        }
        let mut vert_buffer = gl.new_vertex_buffer(vertices.len() as u32, BufferAccess::Static);
        let mut index_buffer = gl.new_index_buffer(indices.len() as u32, BufferAccess::Static);
        vert_buffer.data(0, &vertices);
        index_buffer.data(0, &indices);
        Model::<V> {
            vertices: vert_buffer,
            indices: index_buffer,
        }
    }
}

pub struct Audio 
{
    channels: u16,
    sample_rate: u32,
    duration: Duration,
    data: Arc<Vec<f32>>
}

pub struct AudioSource
{
    channels: u16,
    sample_rate: u32,
    duration: Duration,
    data: Arc<Vec<f32>>,
    index: usize
}

impl Load for Audio 
{
    type Config = ();
    fn load(key_gen: &mut Id<u64>, file_path: &std::path::PathBuf, ctx: &mut Context) -> Loadprotocol 
    {
        let mut lp = Loadprotocol::empty(format!("Sound {file_path:?}"));
        lp.request_file(key_gen, &file_path.to_string_lossy(), "file", ctx);
        lp
    }

    fn interpret(lp: &Loadprotocol, _gl: &mut Gl, _: &mut Self::Config) -> Self 
    {
        let decoder = Decoder::new_vorbis(std::io::Cursor::new(lp.get_data("file").to_owned())).unwrap(); //TODO to_owned necessary since rodio 0.15.0. avoidable?
        let channels = decoder.channels();
        let sample_rate = decoder.sample_rate();
        let data = decoder.convert_samples::<f32>().collect::<Vec<_>>();
        Audio::new(channels, sample_rate, data)
    }

    fn path(file_name: &'static str) -> std::path::PathBuf 
    {
        PathBuf::from("sounds").join(file_name).with_extension("ogg")
    }
}

impl Audio {
    pub fn buffer(&self) -> AudioSource
    {
        AudioSource { channels: self.channels, sample_rate: self.sample_rate, duration: self.duration, data: self.data.clone(), index: 0 }
    }

    fn new(channels: u16, sample_rate: u32, data: Vec<f32>) -> Self {
        //copied from rodio
        assert!(channels != 0);
        assert!(sample_rate != 0);

        let duration_ns = 1_000_000_000u64.checked_mul(data.len() as u64).unwrap()
            / sample_rate as u64
            / channels as u64;
        let duration = Duration::new(
            duration_ns / 1_000_000_000,
            (duration_ns % 1_000_000_000) as u32,
        );

        Audio {
            channels,
            sample_rate,
            duration,
            data: Arc::new(data)
        }
    }
}

impl Iterator for AudioSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.index;
        self.index += 1;
        self.data.get(i).cloned()
    }
}

impl Source for AudioSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        Some(self.duration)
    }
}
