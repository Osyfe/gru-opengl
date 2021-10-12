use super::*;

impl Gl
{
	pub fn new_vertex_buffer<T: AttributesReprCpacked>(&mut self, length: u32, access: BufferAccess) -> VertexBuffer<T>
	{
		let gl = &self.raw;
		let buffer = unsafe
		{
			let buffer = gl.create_buffer().unwrap();
			gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));
			gl.buffer_data_size(glow::ARRAY_BUFFER, length as i32 * std::mem::size_of::<T>() as i32, access.draw());
			gl.bind_buffer(glow::ARRAY_BUFFER, None);
			buffer
		};
		let size_of_t: usize = T::ATTRIBUTES.iter().map(|(ty, _)| match ty
		{
			BufferType::Float { size } => *size as usize * 4,
			BufferType::Int { size, .. } => *size as usize * 4
		}).sum();
		if size_of_t != std::mem::size_of::<T>() { panic!("Gl::new_vertex_buffer: Wrong attribute trait implementation (the struct size does not match)."); }
		VertexBuffer { gl: gl.clone(), buffer, length, _phantom: PhantomData }
	}

	pub fn new_index_buffer(&mut self, length: u32, access: BufferAccess) -> IndexBuffer
	{
		let gl = &self.raw;
		unsafe
		{
			let buffer = gl.create_buffer().unwrap();
			gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(buffer));
			gl.buffer_data_size(glow::ELEMENT_ARRAY_BUFFER, length as i32 * 2, access.draw());
			gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
			IndexBuffer { gl: gl.clone(), buffer, length }
		}
	}
}

#[derive(Clone, Copy)]
pub enum BufferType
{
	Float { size: u8 },
	Int { signed: bool, size: u8 }
}

pub trait AttributesReprCpacked
{
    const ATTRIBUTES: &'static [(BufferType, &'static str)];
}

#[derive(Clone, Copy)]
pub enum BufferAccess
{
	Static,
	Stream,
	Dynamic
}

impl BufferAccess
{
	const fn draw(&self) -> u32
	{
		match self
		{
			Self::Static => glow::STATIC_DRAW,
			Self::Stream => glow::STREAM_DRAW,
			Self::Dynamic => glow::DYNAMIC_DRAW
		}
	}
}

impl<T: AttributesReprCpacked> VertexBuffer<T>
{
	#[inline]
	pub fn len(&self) -> u32
	{
		self.length
	}

	#[inline]
	pub fn data(&mut self, offset: u32, data: &[T])
	{
		if offset + data.len() as u32 > self.length { panic!("VertexBuffer::data: Too much data."); }
		let gl = &self.gl;
		unsafe
		{
			let ptr = data.as_ptr() as *const u8;
			let data = std::slice::from_raw_parts(ptr, data.len() * std::mem::size_of::<T>());
			gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.buffer));
			gl.buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, offset as i32 * std::mem::size_of::<T>() as i32, data);
			gl.bind_buffer(glow::ARRAY_BUFFER, None);
		}
	}
}

impl IndexBuffer
{
	#[inline]
	pub fn len(&self) -> u32
	{
		self.length
	}
	
	#[inline]
	pub fn data(&mut self, offset: u32, data: &[u16])
	{
		if offset + data.len() as u32 > self.length { panic!("IndexBuffer::data: Too much data."); }
		let gl = &self.gl;
		unsafe
		{
			let ptr = data.as_ptr() as *const u8;
			let data = std::slice::from_raw_parts(ptr, data.len() * 2);
			gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.buffer));
			gl.buffer_sub_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, offset as i32 * 2, data);
			gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
		}
	}
}
