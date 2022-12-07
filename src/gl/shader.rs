use super::*;
use gru_misc::math::*;

impl Gl
{
	pub fn new_shader<T: AttributesReprCpacked>(&mut self, vertex_glsl: &str, fragment_glsl: &str) -> Shader<T>
	{
		let gl = &self.raw;
		let program = unsafe { gl.create_program() }.unwrap();
		let id = self.shader_id;
		self.shader_id += 1;
		//shader
		let vertex_shader = unsafe
		{
			let source = format!("{}\n{}", self.glsl_vertex_header, vertex_glsl);
			let shader = gl.create_shader(glow::VERTEX_SHADER).unwrap();
			gl.shader_source(shader, &source);
			gl.compile_shader(shader);
			if !gl.get_shader_compile_status(shader)
			{
				let info = gl.get_shader_info_log(shader);
				log(&source);
				log(&info);
				panic!("{}", info);
			}
			gl.attach_shader(program, shader);
			shader
		};
		let fragment_shader = unsafe
		{
			let source = format!("{}\n{}", self.glsl_fragment_header, fragment_glsl);
			let shader = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
			gl.shader_source(shader, &source);
			gl.compile_shader(shader);
			if !gl.get_shader_compile_status(shader)
			{
				let info = gl.get_shader_info_log(shader);
				log(&source);
				log(&info);
				panic!("{}", info);
			}
			gl.attach_shader(program, shader);
			shader
		};
		let mut attributes = Vec::new();
		let mut uniforms = AHashMap::new();
		unsafe
		{
			//1. link
			gl.link_program(program);
	        if !gl.get_program_link_status(program)
			{
				let info = gl.get_program_info_log(program);
				log(&info);
				panic!("{}", info);
			}
	        //extract attributes
			let len = gl.get_active_attributes(program);
			for i in 0..len
			{
				let attribute = gl.get_active_attribute(program, i).unwrap();
				Self::attribute_location(&mut self.attributes, &attribute.name, &mut |name, location|
				{
					attributes.push((name.to_string(), attribute.atype));
					gl.bind_attrib_location(program, location, name);
				});
			}
			//validate attributes
			if T::ATTRIBUTES.len() != attributes.len() { log("Wrong number of attributes."); } //no panic due to nVidia attribute elision
			for (buffer_type, name) in T::ATTRIBUTES
			{
				match attributes.iter().find(|attr| &attr.0 == name)
				{
					Some(attr) => if attr.1 != buffer_type.code() { panic!("Wrong attribute type for \"{}\".", attr.0); },
					None => log(&format!("The Shader is missing attribute \"{}\"", name)) //no panic due to nVidia attribute elision
				}
			}
			//2. link
			gl.link_program(program);
	        if !gl.get_program_link_status(program)
			{
				let info = gl.get_program_info_log(program);
				log(&info);
				panic!("{}", info);
			}
			//clean
			gl.detach_shader(program, vertex_shader);
	        gl.delete_shader(vertex_shader);
	        gl.detach_shader(program, fragment_shader);
	        gl.delete_shader(fragment_shader);
			//extract uniforms
			let len = gl.get_active_uniforms(program);
			for i in 0..len
			{
				let uniform = gl.get_active_uniform(program, i).unwrap();
				let location = gl.get_uniform_location(program, &uniform.name).unwrap();
				uniforms.insert(uniform.name, (location, uniform.utype));
			}
		}
		//transform attributes
		let mut attributes = Vec::with_capacity(T::ATTRIBUTES.len());
		let mut size_of_t = 0;
		for (ty, name) in T::ATTRIBUTES
		{
			let mut location = 0;
			Self::attribute_location(&mut self.attributes, name, &mut |_, loc| location = loc);
			attributes.push((*ty, location, size_of_t as i32));
			size_of_t += (match ty
			{
				BufferType::Float { size } => *size,
				#[cfg(not(target_arch = "wasm32"))]
				BufferType::Int { size, .. } => *size
			}) as usize * 4;
		}
        Shader { gl: gl.clone(), id, program, uniforms, attributes, _phantom: PhantomData }
	}
}

impl<T: AttributesReprCpacked> Shader<T>
{
	pub fn get_key<U: UniformType>(&self, name: &str) -> UniformKey<U>
	{
		let (location, utype) = self.uniforms.get(name).expect(&format!("The uniform \"{}\" does not exist.", name));
		if *utype != U::CODE { panic!("The uniform \"{}\" has the wrong type.", name); }
		UniformKey { key: location.clone(), shader_id: self.id, _phatom: PhantomData }
	}
}

pub unsafe trait UniformType: Sized
{
	const CODE: u32;
	unsafe fn set<T: AttributesReprCpacked>(&self, pipeline: &mut Pipeline<T>, key: &UniformKey<Self>);
}

unsafe impl UniformType for f32
{
	const CODE: u32 = glow::FLOAT;
	#[inline]
	unsafe fn set<T: AttributesReprCpacked>(&self, pipeline: &mut Pipeline<T>, key: &UniformKey<Self>) { pipeline.gl.raw.uniform_1_f32(Some(&key.key), *self); }
}

unsafe impl UniformType for Vec2
{
	const CODE: u32 = glow::FLOAT_VEC2;
	#[inline]
	unsafe fn set<T: AttributesReprCpacked>(&self, pipeline: &mut Pipeline<T>, key: &UniformKey<Self>) { pipeline.gl.raw.uniform_2_f32(Some(&key.key), self.0, self.1); }
}

unsafe impl UniformType for Vec3
{
	const CODE: u32 = glow::FLOAT_VEC3;
	#[inline]
	unsafe fn set<T: AttributesReprCpacked>(&self, pipeline: &mut Pipeline<T>, key: &UniformKey<Self>) { pipeline.gl.raw.uniform_3_f32(Some(&key.key), self.0, self.1, self.2); }
}

unsafe impl UniformType for Vec4
{
	const CODE: u32 = glow::FLOAT_VEC4;
	#[inline]
	unsafe fn set<T: AttributesReprCpacked>(&self, pipeline: &mut Pipeline<T>, key: &UniformKey<Self>) { pipeline.gl.raw.uniform_4_f32(Some(&key.key), self.0, self.1, self.2, self.3); }
}

unsafe impl UniformType for i32
{
	const CODE: u32 = glow::INT;
	#[inline]
	unsafe fn set<T: AttributesReprCpacked>(&self, pipeline: &mut Pipeline<T>, key: &UniformKey<Self>) { pipeline.gl.raw.uniform_1_i32(Some(&key.key), *self); }
}

unsafe impl UniformType for (i32, i32)
{
	const CODE: u32 = glow::INT_VEC2;
	#[inline]
	unsafe fn set<T: AttributesReprCpacked>(&self, pipeline: &mut Pipeline<T>, key: &UniformKey<Self>) { pipeline.gl.raw.uniform_2_i32(Some(&key.key), self.0, self.1); }
}

unsafe impl UniformType for (i32, i32, i32)
{
	const CODE: u32 = glow::INT_VEC3;
	#[inline]
	unsafe fn set<T: AttributesReprCpacked>(&self, pipeline: &mut Pipeline<T>, key: &UniformKey<Self>) { pipeline.gl.raw.uniform_3_i32(Some(&key.key), self.0, self.1, self.2); }
}

unsafe impl UniformType for (i32, i32, i32, i32)
{
	const CODE: u32 = glow::INT_VEC4;
	#[inline]
	unsafe fn set<T: AttributesReprCpacked>(&self, pipeline: &mut Pipeline<T>, key: &UniformKey<Self>) { pipeline.gl.raw.uniform_4_i32(Some(&key.key), self.0, self.1, self.2, self.3); }
}

unsafe impl UniformType for u32
{
	const CODE: u32 = glow::UNSIGNED_INT;
	#[inline]
	unsafe fn set<T: AttributesReprCpacked>(&self, pipeline: &mut Pipeline<T>, key: &UniformKey<Self>) { pipeline.gl.raw.uniform_1_u32(Some(&key.key), *self); }
}

unsafe impl UniformType for (u32, u32)
{
	const CODE: u32 = glow::UNSIGNED_INT_VEC2;
	#[inline]
	unsafe fn set<T: AttributesReprCpacked>(&self, pipeline: &mut Pipeline<T>, key: &UniformKey<Self>) { pipeline.gl.raw.uniform_2_u32(Some(&key.key), self.0, self.1); }
}

unsafe impl UniformType for (u32, u32, u32)
{
	const CODE: u32 = glow::UNSIGNED_INT_VEC3;
	#[inline]
	unsafe fn set<T: AttributesReprCpacked>(&self, pipeline: &mut Pipeline<T>, key: &UniformKey<Self>) { pipeline.gl.raw.uniform_3_u32(Some(&key.key), self.0, self.1, self.2); }
}

unsafe impl UniformType for (u32, u32, u32, u32)
{
	const CODE: u32 = glow::UNSIGNED_INT_VEC4;
	#[inline]
	unsafe fn set<T: AttributesReprCpacked>(&self, pipeline: &mut Pipeline<T>, key: &UniformKey<Self>) { pipeline.gl.raw.uniform_4_u32(Some(&key.key), self.0, self.1, self.2, self.3); }
}

unsafe impl UniformType for Mat2
{
	const CODE: u32 = glow::FLOAT_MAT2;
	#[inline]
	unsafe fn set<T: AttributesReprCpacked>(&self, pipeline: &mut Pipeline<T>, key: &UniformKey<Self>) { pipeline.gl.raw.uniform_matrix_2_f32_slice(Some(&key.key), false, &self.to_array()); }
}

unsafe impl UniformType for Mat3
{
	const CODE: u32 = glow::FLOAT_MAT3;
	#[inline]
	unsafe fn set<T: AttributesReprCpacked>(&self, pipeline: &mut Pipeline<T>, key: &UniformKey<Self>) { pipeline.gl.raw.uniform_matrix_3_f32_slice(Some(&key.key), false, &self.to_array()); }
}

unsafe impl UniformType for Mat4
{
	const CODE: u32 = glow::FLOAT_MAT4;
	#[inline]
	unsafe fn set<T: AttributesReprCpacked>(&self, pipeline: &mut Pipeline<T>, key: &UniformKey<Self>) { pipeline.gl.raw.uniform_matrix_4_f32_slice(Some(&key.key), false, &self.to_array()); }
}

unsafe impl<const P: bool> UniformType for Texture<P>
{
	const CODE: u32 = glow::SAMPLER_2D;
	#[inline]
	unsafe fn set<T: AttributesReprCpacked>(&self, pipeline: &mut Pipeline<T>, key: &UniformKey<Self>)
	{
		if let Some(id) = (0..8).map(|id| (id + pipeline.texture_active) % 8).filter(|id| pipeline.texture_lock & (1 << id) == 0).next()
		{
			pipeline.gl.raw.uniform_1_i32(Some(&key.key), id as i32);
			pipeline.gl.raw.active_texture(glow::TEXTURE0 + id as u32);
			pipeline.gl.raw.bind_texture(glow::TEXTURE_2D, Some(self.texture));
			pipeline.texture_active = (id + 1) % 8;
			if P { pipeline.texture_lock |= 1 << id; }
		}
		pipeline.texture_used = true;
	}
}
