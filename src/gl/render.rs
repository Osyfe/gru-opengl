use super::*;
use crate::DEBUG;

macro_rules! gl_able
{
	($gl: ident, $info: ident, $self: expr, $field: ident, $gl_name: ident) =>
	{
		if $info.$field && !$self.$field
		{
			unsafe { $gl.enable(glow::$gl_name); }
			$self.$field = true;
		}
		if !$info.$field && $self.$field
		{
			unsafe { $gl.disable(glow::$gl_name); }
			$self.$field = false;
		}
	}
}

pub struct RenderPassInfo
{
	pub clear_color: Option<(f32, f32, f32)>,
	pub clear_depth: bool
}

#[derive(Clone, Copy)]
pub struct PipelineInfo
{
	pub depth_test: bool,
	pub alpha_blend: bool,
	pub face_cull: bool
}

#[derive(Clone, Copy)]
pub enum Primitives
{
	Points,
	Lines,
	LineStrip,
	Triangles
}

impl Primitives
{
	const fn gl_name(&self) -> u32
	{
		match self
		{
			Self::Points => glow::POINTS,
			Self::Lines => glow::LINES,
			Self::LineStrip => glow::LINE_STRIP,
			Self::Triangles => glow::TRIANGLES
		}
	}
}

impl Gl
{
	#[inline]
	pub fn render_pass<'a, 'b>(&'a mut self, render_target: RenderTarget<'b>, info: RenderPassInfo) -> RenderPass<'a, 'b>
	{
		let gl = &self.raw;
		let (width, height) = match &render_target
		{
			RenderTarget::Screen => (self.window_dims.0 as i32, self.window_dims.1 as i32),
			RenderTarget::Texture(framebuffer) =>
			{
				unsafe { gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer.framebuffer)); }
				(framebuffer.size() as i32, framebuffer.size() as i32)
			}
		};
		if (width, height) != self.viewport
		{
			unsafe { gl.viewport(0, 0, width, height); }
			self.viewport = (width, height);
		}
		if let Some(clear_color) = info.clear_color
		{
			if Some(clear_color) != Some(self.clear_color)
			{
				unsafe { gl.clear_color(clear_color.0, clear_color.1, clear_color.2, 1.0); }
				self.clear_color = clear_color;
			}
			unsafe { gl.clear(if info.clear_depth { glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT } else { glow::COLOR_BUFFER_BIT }); }
		} else if info.clear_depth { unsafe { gl.clear(glow::DEPTH_BUFFER_BIT); } }
		RenderPass { gl: self, render_target }
	}
}

impl<'a, 'b> RenderPass<'a, 'b>
{
	#[inline]
	pub fn pipeline<'c, 'd, T: AttributesReprCpacked>(&'c mut self, shader: &'d Shader<T>, info: PipelineInfo) -> Pipeline<'c, 'd, T>
	{
		let gl = &self.gl.raw;
		gl_able!(gl, info, self.gl.pipeline, depth_test, DEPTH_TEST);
		gl_able!(gl, info, self.gl.pipeline, alpha_blend, BLEND);
		gl_able!(gl, info, self.gl.pipeline, face_cull, CULL_FACE);
		unsafe { gl.use_program(Some(shader.program)); }
		Pipeline { gl: &mut self.gl, shader, texture_active: 0, texture_lock: 0, texture_used: false }
	}
}

impl<'a, 'b, T: AttributesReprCpacked> Pipeline<'a, 'b, T>
{
	#[inline]
	pub fn uniform_name<U: UniformType>(&mut self, name: &str, value: &U) -> &mut Self
	{
		let key = self.shader.get_key(name);
		unsafe { value.set(self, &key); }
		self
	}

	#[inline]
	pub fn uniform_key<U: UniformType>(&mut self, key: &UniformKey<U>, value: &U) -> &mut Self
	{
		if DEBUG && self.shader.id != key.shader_id
		{
			let msg = "The Shader and UniformKey are incompatible.";
			log(msg);
			panic!("{}", msg);
		}
		unsafe { value.set(self, key); }
		self
	}

	#[inline]
	pub fn draw(&mut self, primitives: Primitives, vertices: &VertexBuffer<T>, indices: Option<&IndexBuffer>, offset: u32, count: u32)
	{
		let gl = &self.gl.raw;
		unsafe
		{
			gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertices.buffer));

			for (ty, location, offset) in &self.shader.attributes
			{
				match ty
				{
					BufferType::Float { size } => gl.vertex_attrib_pointer_f32(*location, *size as i32, glow::FLOAT, false, std::mem::size_of::<T>() as i32, *offset),
					BufferType::Int { signed, size } => gl.vertex_attrib_pointer_i32(*location, *size as i32, if *signed { glow::INT } else { glow::UNSIGNED_INT }, std::mem::size_of::<T>() as i32, *offset)
				}
				gl.enable_vertex_attrib_array(*location);
			}

			match indices
			{
				None =>
				{
					if offset + count > vertices.length { panic!("Pipeline::draw: Not enough vertices in buffer."); }
					gl.draw_arrays(primitives.gl_name(), offset as i32, count as i32);
				},
				Some(indices) =>
				{
					if offset + count > indices.length { panic!("Pipeline::draw: Not enough indices in buffer."); }
					gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(indices.buffer));
					gl.draw_elements(primitives.gl_name(), count as i32, glow::UNSIGNED_SHORT, offset as i32);
					gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
				}
			}
			
			for (_, location, _) in &self.shader.attributes { gl.disable_vertex_attrib_array(*location); }

			gl.bind_buffer(glow::ARRAY_BUFFER, None);
		}
	}
}

impl<T: AttributesReprCpacked> Drop for Pipeline<'_, '_, T>
{
	#[inline]
	fn drop(&mut self)
	{
		let gl = &self.gl.raw;
		unsafe
		{
			if self.texture_used { gl.bind_texture(glow::TEXTURE_2D, None); }
			gl.use_program(None);
		}
	}
}

impl Drop for RenderPass<'_, '_>
{
	#[inline]
	fn drop(&mut self)
	{
		if let RenderTarget::Texture(_) = self.render_target { unsafe { self.gl.raw.bind_framebuffer(glow::FRAMEBUFFER, None); } }
	}
}
