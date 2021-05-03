use super::*;

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
				Self::attribute_location(&mut self.attributes, &attribute.name, &mut |name, location| gl.bind_attrib_location(program, location, name));
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
				uniforms.insert(uniform.name, UniformKey(location));
			}
		}
        Shader { gl: gl.clone(), program, uniforms }
	}
}

impl Shader
{
	pub fn get_key(&self, name: &str) -> Option<&UniformKey>
	{
		self.uniforms.get(name)
	}
}
