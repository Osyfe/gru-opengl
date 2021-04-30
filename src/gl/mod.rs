use super::log;
use gru_math::*;
use glow::{Context, HasContext};
use std::marker::PhantomData;
use std::rc::Rc;
use std::collections::HashMap;

mod drops;
mod buffer;
mod render;
pub use buffer::*;
pub use render::*;

pub struct Gl
{
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
			raw: Rc::new(gl),
			glsl_vertex_header,
			glsl_fragment_header,
			viewport: (0, 0),
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
			gl.disable(glow::BLEND);
			gl.blend_equation(glow::FUNC_ADD);
			gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
			gl.enable(glow::CULL_FACE);
			gl.cull_face(glow::BACK);
		}
		self
	}

	pub(crate) fn viewport(&mut self, (width, height): (u32, u32))
	{
		let (width, height) = (width as i32, height as i32);
		if self.viewport != (width, height)
		{
			unsafe { self.raw.viewport(0, 0, width, height); }
			self.viewport = (width, height);
		}
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
	length: u32
}
//u16
pub struct IndexBuffer
{
	gl: Rc<Context>,
	buffer: <Context as HasContext>::Buffer,
	length: u32
}

pub struct Shader
{
	gl: Rc<Context>,
	program: <Context as HasContext>::Program,
	uniforms: HashMap<String, <Context as HasContext>::UniformLocation>
}
