use super::log;
use glow::{Context, HasContext};
use std::{rc::Rc, marker::PhantomData};
use ahash::AHashMap;

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
	shader_id: u32,
	viewport: (i32, i32),
	clear_color: (f32, f32, f32),
	attributes: AHashMap<String, u32>,
	pipeline: PipelineInfo
}

impl Gl
{
	pub(crate) fn new(gl: Context, glsl_vertex_header: &'static str, glsl_fragment_header: &'static str) -> Self
	{
		unsafe
		{
			#[cfg(not(target_arch = "wasm32"))]
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

		Self
		{
			window_dims: (0, 0),
			raw: Rc::new(gl),
			glsl_vertex_header,
			glsl_fragment_header,
			shader_id: 0,
			viewport: (-1, -1),
			clear_color: (0.0, 0.0, 0.0),
			attributes: AHashMap::new(),
			pipeline: PipelineInfo
			{
				depth_test: true,
				alpha_blend: false,
				face_cull: true
			}
		}
	}

	fn attribute_location(attributes: &mut AHashMap<String, u32>, name: &str, action: &mut dyn FnMut(&str, u32))
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
	length: u32,
	_phantom: PhantomData<T>
}
//u16 indices
pub struct IndexBuffer
{
	gl: Rc<Context>,
	buffer: <Context as HasContext>::Buffer,
	length: u32
}
//P: texture locks uniform location during the entire pipeline (8 in total!)
pub struct Texture<const P: bool>
{
	gl: Rc<Context>,
	texture: <Context as HasContext>::Texture,
	size: u32
}

pub struct Shader<T: AttributesReprCpacked>
{
	gl: Rc<Context>,
	id: u32,
	program: <Context as HasContext>::Program,
	uniforms: AHashMap<String, (<Context as HasContext>::UniformLocation, u32)>, //(shader name, opengl location, glow type)
	attributes: Vec<(BufferType, u32, i32)>, //(gru type, location, offset)
	_phantom: PhantomData<T>
}

pub struct Framebuffer
{
	gl: Rc<Context>,
	framebuffer: <Context as HasContext>::Framebuffer,
	color: Texture<true>,
	depth: Option<<Context as HasContext>::Renderbuffer>
}

pub struct UniformKey<U: shader::UniformType>
{
	key: <Context as HasContext>::UniformLocation,
	shader_id: u32,
	_phatom: PhantomData<U>
}

pub enum RenderTarget<'a>
{
	Screen,
	Texture(&'a mut Framebuffer)
}

pub struct RenderPass<'a, 'b>
{
	gl: &'a mut Gl,
	render_target: RenderTarget<'b>
}

pub struct Pipeline<'a, 'b, T: AttributesReprCpacked>
{
	gl: &'a mut Gl,
	shader: &'b Shader<T>,
	texture_active: u8,
	texture_lock: u8,
	texture_used: bool
}
