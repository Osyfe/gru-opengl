use super::*;

impl Gl
{
	pub fn new_texture<const P: bool>(&mut self, TextureConfig { size, channel, mipmap, wrap_s, wrap_t }: &TextureConfig, data: &[u8]) -> Texture<P>
	{
		if size & (size - 1) != 0 { panic!("Gl::new_texture: Size is not a power of 2."); }
		if size.pow(2) * channel.bytes() != data.len() as u32 { panic!("Gl::new_texture: Data has the wrong length."); }
		let gl = &self.raw;
		unsafe
		{
			let texture = gl.create_texture().unwrap();
			gl.bind_texture(glow::TEXTURE_2D, Some(texture));
			gl.tex_image_2d(glow::TEXTURE_2D, 0, channel.format() as i32, *size as i32, *size as i32, 0, channel.format(), glow::UNSIGNED_BYTE, Some(data));
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, wrap_s.wrap() as i32);
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, wrap_t.wrap() as i32);
			if *mipmap
			{
				gl.generate_mipmap(glow::TEXTURE_2D);
				gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR_MIPMAP_LINEAR as i32);
			} else { gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32); }
			gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
			gl.bind_texture(glow::TEXTURE_2D, None);
			Texture { gl: gl.clone(), texture, size: *size }
		}
	}
}

#[derive(Clone, Copy)]
pub enum TextureChannel
{
	A,
	RGB,
	RGBA
}

impl TextureChannel
{
	fn bytes(&self) -> u32
	{
		match self
		{
			Self::A => 1,
			Self::RGB => 3,
			Self::RGBA => 4
		}
	}

	fn format(&self) -> u32
	{
		match self
		{
			Self::A => glow::ALPHA,
			Self::RGB => glow::RGB,
			Self::RGBA => glow::RGBA
		}
	}
}

#[derive(Clone, Copy)]
pub enum TextureWrap
{
	Clamp,
	Repeat
}

impl TextureWrap
{
	pub(crate) fn wrap(&self) -> u32
	{
		match self
		{
			Self::Clamp => glow::CLAMP_TO_EDGE,
			Self::Repeat => glow::REPEAT
		}
	}
}

#[derive(Clone)]
pub struct TextureConfig
{
	pub size: u32,
	pub channel: TextureChannel,
	pub mipmap: bool,
	pub wrap_s: TextureWrap,
    pub wrap_t: TextureWrap
}

impl<const P: bool> Texture<P>
{
	pub fn size(&self) -> u32
	{
		self.size
	}
}
