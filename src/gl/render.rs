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

pub enum RenderTarget<'a>
{
	Screen,
	Texture(&'a mut Framebuffer)
}

pub struct RenderPassInfo
{
	pub clear_color: Option<(f32, f32, f32)>,
	pub clear_depth: bool
}

pub struct RenderPass<'a, 'b>
{
	gl: &'a mut Gl,
	render_target: RenderTarget<'b>
}

#[derive(Clone, Copy)]
pub struct PipelineInfo
{
	pub depth_test: bool,
	pub alpha_blend: bool,
	pub face_cull: bool
}

pub struct Pipeline<'a, 'b, T: AttributesReprCpacked>
{
	gl: &'a mut Gl,
	shader: &'b Shader<T>,
	texture_active: u8,
	texture_lock: u8,
	texture_used: bool
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
	pub fn shader(&self) -> &Shader<T>
	{
		&self.shader
	}

	#[inline(always)]
	fn check_key(shader: &Shader<T>, key: &UniformKey)
	{
		if DEBUG && shader.id != key.shader_id
		{
			let msg = "The Shader and UniformKey are incompatible.";
			log(msg);
			panic!("{}", msg);
		}
	}

	#[inline]
	pub fn uniform_f1(&mut self, key: &UniformKey, value: f32) -> &mut Self
	{
		Self::check_key(&self.shader, key);
		unsafe { self.gl.raw.uniform_1_f32(Some(&key.key), value); }
		self
	}

	#[inline]
	pub fn uniform_f2(&mut self, key: &UniformKey, value: Vec2) -> &mut Self
	{
		Self::check_key(&self.shader, key);
		unsafe { self.gl.raw.uniform_2_f32(Some(&key.key), value.0, value.1); }
		self
	}

	#[inline]
	pub fn uniform_f3(&mut self, key: &UniformKey, value: Vec3) -> &mut Self
	{
		Self::check_key(&self.shader, key);
		unsafe { self.gl.raw.uniform_3_f32(Some(&key.key), value.0, value.1, value.2); }
		self
	}

	#[inline]
	pub fn uniform_f4(&mut self, key: &UniformKey, value: Vec4) -> &mut Self
	{
		Self::check_key(&self.shader, key);
		unsafe { self.gl.raw.uniform_4_f32(Some(&key.key), value.0, value.1, value.2, value.3); }
		self
	}

	#[inline]
	pub fn uniform_i1(&mut self, key: &UniformKey, v0: i32) -> &mut Self
	{
		Self::check_key(&self.shader, key);
		unsafe { self.gl.raw.uniform_1_i32(Some(&key.key), v0); }
		self
	}

	#[inline]
	pub fn uniform_i2(&mut self, key: &UniformKey, (v0, v1): (i32, i32)) -> &mut Self
	{
		Self::check_key(&self.shader, key);
		unsafe { self.gl.raw.uniform_2_i32(Some(&key.key), v0, v1); }
		self
	}

	#[inline]
	pub fn uniform_i3(&mut self, key: &UniformKey, (v0, v1, v2): (i32, i32, i32)) -> &mut Self
	{
		Self::check_key(&self.shader, key);
		unsafe { self.gl.raw.uniform_3_i32(Some(&key.key), v0, v1, v2); }
		self
	}

	#[inline]
	pub fn uniform_i4(&mut self, key: &UniformKey, (v0, v1, v2, v3): (i32, i32, i32, i32)) -> &mut Self
	{
		Self::check_key(&self.shader, key);
		unsafe { self.gl.raw.uniform_4_i32(Some(&key.key), v0, v1, v2, v3); }
		self
	}

	#[inline]
	pub fn uniform_u1(&mut self, key: &UniformKey, v0: u32) -> &mut Self
	{
		Self::check_key(&self.shader, key);
		unsafe { self.gl.raw.uniform_1_u32(Some(&key.key), v0); }
		self
	}

	#[inline]
	pub fn uniform_u2(&mut self, key: &UniformKey, (v0, v1): (u32, u32)) -> &mut Self
	{
		Self::check_key(&self.shader, key);
		unsafe { self.gl.raw.uniform_2_u32(Some(&key.key), v0, v1); }
		self
	}

	#[inline]
	pub fn uniform_u3(&mut self, key: &UniformKey, (v0, v1, v2): (u32, u32, u32)) -> &mut Self
	{
		Self::check_key(&self.shader, key);
		unsafe { self.gl.raw.uniform_3_u32(Some(&key.key), v0, v1, v2); }
		self
	}

	#[inline]
	pub fn uniform_u4(&mut self, key: &UniformKey, (v0, v1, v2, v3): (u32, u32, u32, u32)) -> &mut Self
	{
		Self::check_key(&self.shader, key);
		unsafe { self.gl.raw.uniform_4_u32(Some(&key.key), v0, v1, v2, v3); }
		self
	}

	#[inline]
	pub fn uniform_mat2(&mut self, key: &UniformKey, value: Mat2) -> &mut Self
	{
		Self::check_key(&self.shader, key);
		unsafe { self.gl.raw.uniform_matrix_2_f32_slice(Some(&key.key), false, &value.to_array()); }
		self
	}

	#[inline]
	pub fn uniform_mat3(&mut self, key: &UniformKey, value: Mat3) -> &mut Self
	{
		Self::check_key(&self.shader, key);
		unsafe { self.gl.raw.uniform_matrix_3_f32_slice(Some(&key.key), false, &value.to_array()); }
		self
	}

	#[inline]
	pub fn uniform_mat4(&mut self, key: &UniformKey, value: Mat4) -> &mut Self
	{
		Self::check_key(&self.shader, key);
		unsafe { self.gl.raw.uniform_matrix_4_f32_slice(Some(&key.key), false, &value.to_array()); }
		self
	}

	#[inline]
	pub fn uniform_texture(&mut self, key: &UniformKey, value: &Texture, persistent: bool) -> &mut Self
	{
		Self::check_key(&self.shader, key);
		if let Some(id) = (0..8).map(|id| (id + self.texture_active) % 8).filter(|id| self.texture_lock & (1 << id) == 0).next()
		{
			let gl = &self.gl.raw;
			unsafe
			{
				gl.uniform_1_i32(Some(&key.key), id as i32);
				gl.active_texture(glow::TEXTURE0 + id as u32);
				gl.bind_texture(glow::TEXTURE_2D, Some(value.texture));
			}
			self.texture_active = (id + 1) % 8;
			if persistent { self.texture_lock |= 1 << id; }
		}
		self.texture_used = true;
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
