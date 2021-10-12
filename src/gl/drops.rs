use super::*;

impl<T: AttributesReprCpacked> Drop for VertexBuffer<T>
{
	fn drop(&mut self)
	{
		unsafe { self.gl.delete_buffer(self.buffer); }
	}
}

impl Drop for IndexBuffer
{
	fn drop(&mut self)
	{
		unsafe { self.gl.delete_buffer(self.buffer); }
	}
}

impl Drop for Texture
{
	fn drop(&mut self)
	{
		unsafe { self.gl.delete_texture(self.texture); }
	}
}

impl<T: AttributesReprCpacked> Drop for Shader<T>
{
	fn drop(&mut self)
	{
		unsafe { self.gl.delete_program(self.program) };
	}
}

impl Drop for Framebuffer
{
	fn drop(&mut self)
	{
		unsafe
		{
			if let Some(renderbuffer) = self.depth { self.gl.delete_renderbuffer(renderbuffer); }
			self.gl.delete_framebuffer(self.framebuffer);
		}
	}
}
