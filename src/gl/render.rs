use super::*;

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

impl Gl
{
	pub fn new_shader(&mut self, vertex_glsl: &str, fragment_glsl: &str) -> Shader
	{
		let gl = &self.raw;
		let program = unsafe { gl.create_program() }.unwrap();
		//shader
		let vertex_shader = unsafe
		{
			let shader = gl.create_shader(glow::VERTEX_SHADER).unwrap();
			gl.shader_source(shader, &format!("{}\n{}", self.glsl_vertex_header, vertex_glsl));
			gl.compile_shader(shader);
			if !gl.get_shader_compile_status(shader)
			{
				let info = gl.get_shader_info_log(shader);
				log(&info);
				panic!("{}", info);
			}
			gl.attach_shader(program, shader);
			shader
		};
		let fragment_shader = unsafe
		{
			let shader = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
			gl.shader_source(shader, &format!("{}\n{}", self.glsl_fragment_header, fragment_glsl));
			gl.compile_shader(shader);
			if !gl.get_shader_compile_status(shader)
			{
				let info = gl.get_shader_info_log(shader);
				log(&info);
				panic!("{}", info);
			}
			gl.attach_shader(program, shader);
			shader
		};
		let mut uniforms = HashMap::new();
		unsafe
		{
			//link
			gl.link_program(program);
	        if !gl.get_program_link_status(program)
			{
				let info = gl.get_program_info_log(program);
				log(&info);
				panic!("{}", info);
			}
			gl.detach_shader(program, vertex_shader);
	        gl.delete_shader(vertex_shader);
	        gl.detach_shader(program, fragment_shader);
	        gl.delete_shader(fragment_shader);
	        //extract attributes
			let len = gl.get_active_attributes(program);
			for i in 0..len
			{
				let attribute = gl.get_active_attribute(program, i).unwrap();
				if attribute.name.starts_with("gl_") { continue };
				Self::attribute_location(&mut self.attributes, &attribute.name, &mut |name, location|
				{
					gl.bind_attrib_location(program, location, name);
					gl.enable_vertex_attrib_array(location);
				});
			}
			//extract uniforms
			let len = gl.get_active_uniforms(program);
			for i in 0..len
			{
				let uniform = gl.get_active_uniform(program, i).unwrap();
				let location = gl.get_uniform_location(program, &uniform.name).unwrap();
				uniforms.insert(uniform.name, location);
			}
		}
        Shader { gl: gl.clone(), program, uniforms }
	}
	
	//TODO framebuffer binding
	#[inline]
	pub fn render_pass<'a>(&'a mut self, info: RenderPassInfo) -> RenderPass<'a>
	{
		let gl = &self.raw;
		if let Some(clear_color) = info.clear_color
		{
			if Some(clear_color) != Some(self.clear_color)
			{
				unsafe { gl.clear_color(clear_color.0, clear_color.1, clear_color.2, 1.0); }
				self.clear_color = clear_color;
			}
			unsafe { gl.clear(if info.clear_depth { glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT } else { glow::COLOR_BUFFER_BIT }); }
		} else if info.clear_depth { unsafe { gl.clear(glow::DEPTH_BUFFER_BIT); } }
		RenderPass { gl: self }
	}
}

pub struct RenderPassInfo
{
	pub clear_color: Option<(f32, f32, f32)>,
	pub clear_depth: bool
}

pub struct RenderPass<'a>
{
	gl: &'a mut Gl
}

impl<'a> RenderPass<'a>
{
	#[inline]
	pub fn pipeline<'b>(&'b mut self, shader: &'a Shader, info: PipelineInfo) -> Pipeline<'b>
	{
		let gl = &self.gl.raw;
		gl_able!(gl, info, self.gl.pipeline, depth_test, DEPTH_TEST);
		gl_able!(gl, info, self.gl.pipeline, alpha_blend, BLEND);
		gl_able!(gl, info, self.gl.pipeline, face_cull, CULL_FACE);
		unsafe { gl.use_program(Some(shader.program)); }
		Pipeline { gl: &mut self.gl, shader }
	}
}

impl Drop for RenderPass<'_>
{
	#[inline]
	fn drop(&mut self)
	{
		//TODO unbind framebuffer
	}
}

#[derive(Clone, Copy)]
pub struct PipelineInfo
{
	pub depth_test: bool,
	pub alpha_blend: bool,
	pub face_cull: bool
}

pub struct Pipeline<'a>
{
	gl: &'a mut Gl,
	shader: &'a Shader
}

impl<'a> Pipeline<'a>
{
	#[inline]
	pub fn uniform_f1(&mut self, name: &str, value: f32)
	{
		unsafe { self.gl.raw.uniform_1_f32(self.shader.uniforms.get(name), value); }
	}

	#[inline]
	pub fn uniform_f2(&mut self, name: &str, value: Vec2)
	{
		unsafe { self.gl.raw.uniform_2_f32(self.shader.uniforms.get(name), value.0, value.1); }
	}

	#[inline]
	pub fn uniform_f3(&mut self, name: &str, value: Vec3)
	{
		unsafe { self.gl.raw.uniform_3_f32(self.shader.uniforms.get(name), value.0, value.1, value.2); }
	}

	#[inline]
	pub fn uniform_f4(&mut self, name: &str, value: Vec4)
	{
		unsafe { self.gl.raw.uniform_4_f32(self.shader.uniforms.get(name), value.0, value.1, value.2, value.3); }
	}

	#[inline]
	pub fn uniform_i1(&mut self, name: &str, v0: i32)
	{
		unsafe { self.gl.raw.uniform_1_i32(self.shader.uniforms.get(name), v0); }
	}

	#[inline]
	pub fn uniform_i2(&mut self, name: &str, (v0, v1): (i32, i32))
	{
		unsafe { self.gl.raw.uniform_2_i32(self.shader.uniforms.get(name), v0, v1); }
	}

	#[inline]
	pub fn uniform_i3(&mut self, name: &str, (v0, v1, v2): (i32, i32, i32))
	{
		unsafe { self.gl.raw.uniform_3_i32(self.shader.uniforms.get(name), v0, v1, v2); }
	}

	#[inline]
	pub fn uniform_i4(&mut self, name: &str, (v0, v1, v2, v3): (i32, i32, i32, i32))
	{
		unsafe { self.gl.raw.uniform_4_i32(self.shader.uniforms.get(name), v0, v1, v2, v3); }
	}

	#[inline]
	pub fn uniform_u1(&mut self, name: &str, v0: u32)
	{
		unsafe { self.gl.raw.uniform_1_u32(self.shader.uniforms.get(name), v0); }
	}

	#[inline]
	pub fn uniform_u2(&mut self, name: &str, (v0, v1): (u32, u32))
	{
		unsafe { self.gl.raw.uniform_2_u32(self.shader.uniforms.get(name), v0, v1); }
	}

	#[inline]
	pub fn uniform_u3(&mut self, name: &str, (v0, v1, v2): (u32, u32, u32))
	{
		unsafe { self.gl.raw.uniform_3_u32(self.shader.uniforms.get(name), v0, v1, v2); }
	}

	#[inline]
	pub fn uniform_u4(&mut self, name: &str, (v0, v1, v2, v3): (u32, u32, u32, u32))
	{
		unsafe { self.gl.raw.uniform_4_u32(self.shader.uniforms.get(name), v0, v1, v2, v3); }
	}

	#[inline]
	pub fn uniform_mat2(&mut self, name: &str, value: Mat2)
	{
		unsafe { self.gl.raw.uniform_matrix_2_f32_slice(self.shader.uniforms.get(name), false, &value.to_array()); }
	}

	#[inline]
	pub fn uniform_mat3(&mut self, name: &str, value: Mat3)
	{
		unsafe { self.gl.raw.uniform_matrix_3_f32_slice(self.shader.uniforms.get(name), false, &value.to_array()); }
	}

	#[inline]
	pub fn uniform_mat4(&mut self, name: &str, value: Mat4)
	{
		unsafe { self.gl.raw.uniform_matrix_4_f32_slice(self.shader.uniforms.get(name), false, &value.to_array()); }
	}

	#[inline]
	pub fn draw<T: AttributesReprCpacked>(&mut self, vertices: &VertexBuffer<T>, indices: Option<&IndexBuffer>, offset: u32, count: u32)
	{
		let gl = &self.gl.raw;
		unsafe
		{
			gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertices.buffer));
			match indices
			{
				None =>
				{
					if offset + count > vertices.length { panic!("Pipeline::draw: Not enough vertices in buffer."); }
					gl.draw_arrays(glow::TRIANGLES, offset as i32, count as i32);
				},
				Some(indices) =>
				{
					if offset + count > indices.length { panic!("Pipeline::draw: Not enough indices in buffer."); }
					gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(indices.buffer));
					gl.draw_elements(glow::TRIANGLES, count as i32, glow::UNSIGNED_SHORT, offset as i32);
					gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
				}
			}
			gl.bind_buffer(glow::ARRAY_BUFFER, None);
		}
	}
}

impl Drop for Pipeline<'_>
{
	#[inline]
	fn drop(&mut self)
	{
		unsafe { self.gl.raw.use_program(None); }
	}
}
