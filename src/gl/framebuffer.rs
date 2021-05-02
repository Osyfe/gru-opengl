use super::*;

impl Gl
{
	pub fn new_framebuffer(&mut self, FramebufferConfig { depth, size, wrap }: &FramebufferConfig) -> Framebuffer
	{
		if size & (size - 1) != 0 { panic!("Gl::new_framebuffer: Size is not a power of 2."); }
		let gl = &self.raw;
		unsafe
		{
			let framebuffer = gl.create_framebuffer().unwrap();
			gl.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer));

			let color = gl.create_texture().unwrap();
			gl.bind_texture(glow::TEXTURE_2D, Some(color));
			gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::RGBA as i32, *size as i32, *size as i32, 0, glow::RGBA, glow::UNSIGNED_BYTE, None);
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, wrap.wrap() as i32);
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, wrap.wrap() as i32);
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
			gl.bind_texture(glow::TEXTURE_2D, None);
			gl.framebuffer_texture_2d(glow::FRAMEBUFFER, glow::COLOR_ATTACHMENT0, glow::TEXTURE_2D, Some(color), 0);
			let color = Texture { gl: gl.clone(), texture: color, size: *size };

			let depth = if *depth
			{
				let renderbuffer = gl.create_renderbuffer().unwrap();
				gl.bind_renderbuffer(glow::RENDERBUFFER, Some(renderbuffer));
				gl.renderbuffer_storage(glow::RENDERBUFFER, glow::DEPTH_COMPONENT16, *size as i32, *size as i32);
				gl.bind_renderbuffer(glow::RENDERBUFFER, None);
				gl.framebuffer_renderbuffer(glow::FRAMEBUFFER, glow::DEPTH_ATTACHMENT, glow::RENDERBUFFER, Some(renderbuffer));
				Some(renderbuffer)
			} else { None };

			gl.bind_framebuffer(glow::FRAMEBUFFER, None);
			Framebuffer { gl: gl.clone(), framebuffer, color, depth, size: *size }
		}
	}
}

#[derive(Clone, Copy, PartialEq)]
pub enum FramebufferAttachment
{
	Color,
	Depth,
	ColorDepth
}

#[derive(Clone)]
pub struct FramebufferConfig
{
	pub depth: bool,
	pub size: u32,
	pub wrap: TextureWrap
}

impl Framebuffer
{
	pub fn size(&self) -> u32
	{
		self.size
	}
	
	pub fn texture(&self) -> &Texture
	{
		&self.color
	}
}
