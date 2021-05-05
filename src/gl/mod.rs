use super::log;
use gru_math::*;
use glow::{Context, HasContext};
use std::marker::PhantomData;
use std::rc::Rc;
use std::collections::HashMap;

mod drops;
mod buffer;
mod texture;
mod shader;
mod render;
mod framebuffer;
pub use buffer::*;
pub use texture::*;
pub use shader::*;
pub use render::*;
pub use framebuffer::*;

pub struct Gl
{
	pub(crate) window_dims: (u32, u32),
	raw: Rc<Context>,
	glsl_vertex_header: &'static str,
	glsl_fragment_header: &'static str,
	viewport: (i32, i32),
	clear_color: (f32, f32, f32),
	attributes: HashMap<String, u32>,
	pipeline: PipelineInfo
}

impl Gl
{
	pub(crate) fn new(gl: Context, glsl_vertex_header: &'static str, glsl_fragment_header: &'static str) -> Self
	{
		Self
		{
			window_dims: (0, 0),
			raw: Rc::new(gl),
			glsl_vertex_header,
			glsl_fragment_header,
			viewport: (-1, -1),
			clear_color: (0.0, 0.0, 0.0),
			attributes: HashMap::new(),
			pipeline: PipelineInfo
			{
				depth_test: true,
				alpha_blend: false,
				face_cull: true
			}
		}
	}

	pub(crate) fn init(&mut self) -> &mut Self
	{
		let gl = &self.raw;
		unsafe
		{
			#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
			gl.disable(glow::FRAMEBUFFER_SRGB);
			gl.clear_color(0.0, 0.0, 0.0, 1.0);

			gl.enable(glow::DEPTH_TEST);
			gl.depth_func(glow::LEQUAL);
			gl.disable(glow::BLEND);
			gl.blend_equation(glow::FUNC_ADD);
			gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
			gl.enable(glow::CULL_FACE);
			gl.cull_face(glow::BACK);

			gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
		}
		self
	}

	fn attribute_location(attributes: &mut HashMap<String, u32>, name: &str, action: &mut dyn FnMut(&str, u32))
	{
		use std::collections::hash_map::Entry;
		let new_location = attributes.len() as u32;
		match attributes.entry(name.to_string())
		{
			Entry::Vacant(vacant) =>
			{
				action(vacant.key(), new_location);
				vacant.insert(new_location);
			}
			Entry::Occupied(occupied) => action(occupied.key(), *occupied.get())
		}
	}
}

pub struct VertexBuffer<T: AttributesReprCpacked>
{
	gl: Rc<Context>,
	buffer: <Context as HasContext>::Buffer,
	_phantom: PhantomData<T>,
	length: u32,
	attributes: Vec<(BufferType, u32, i32)>
}
//u16
pub struct IndexBuffer
{
	gl: Rc<Context>,
	buffer: <Context as HasContext>::Buffer,
	length: u32
}

pub struct Texture
{
	gl: Rc<Context>,
	texture: <Context as HasContext>::Texture,
	size: u32
}

pub struct Shader
{
	gl: Rc<Context>,
	program: <Context as HasContext>::Program,
	uniforms: HashMap<String, UniformKey>
}

pub struct Framebuffer
{
	gl: Rc<Context>,
	framebuffer: <Context as HasContext>::Framebuffer,
	color: Texture,
	depth: Option<<Context as HasContext>::Renderbuffer>
}

#[derive(Clone)]
pub struct UniformKey(<Context as HasContext>::UniformLocation);
