use super::*;

impl Drop for Shader
{
	fn drop(&mut self)
	{
		unsafe { self.gl.delete_program(self.program) };
	}
}

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
