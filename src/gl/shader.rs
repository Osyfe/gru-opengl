use crate::DEBUG;

use super::*;

impl Gl
{
	pub fn new_shader(&mut self, vertex_glsl: &str, fragment_glsl: &str) -> Shader
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
		let mut attributes = HashSet::new();
		let mut uniforms = HashMap::new();
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
					attributes.insert(name.to_string());
					gl.bind_attrib_location(program, location, name);
				});
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
				uniforms.insert(uniform.name, UniformKey { key: location, shader_id: id });
			}
		}
        Shader { gl: gl.clone(), program, attributes, uniforms, id }
	}
}

impl Shader
{
	pub fn check_attributes<T: AttributesReprCpacked>(&self)
	{
		if DEBUG
		{
			if T::ATTRIBUTES.len() != self.attributes.len()
			{
				let msg = "Wrong number of attributes.";
				log(msg);
				panic!("{}", msg);
			}
			for (_, name) in T::ATTRIBUTES
			{
				if !self.attributes.contains(*name)
				{
					let msg = format!("The Shader has no attribute \"{}\"", name);
					log(&msg);
					panic!("{}", msg);
				}
			}
		}
	}

	pub fn get_key(&self, name: &str) -> Option<&UniformKey>
	{
		self.uniforms.get(name)
	}
}
