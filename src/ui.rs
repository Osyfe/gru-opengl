use gru_misc::ui::*;

use crate::event::{Event as GlEvent, MouseButton as GlButton, KeyCode as GlKey, Scroll as GlScroll};
use event::{Event as UiEvent, MouseButton as UiButton, Key as UiKey};
use gru_misc::paint::{Vec2, Frame as PaintFrame, TEXTURE_SIZE};
use crate::gl::*;

const VERT: &str =
"
    attribute vec2 in_pos;
    attribute vec2 in_coords;
    attribute vec4 in_color;

    varying vec2 out_coords;
    varying vec4 out_color;

    void main()
    {
        out_coords = in_coords;
        out_color = in_color;
        gl_Position = vec4(in_pos, 0.0, 1.0);
    }
";

const FRAG: &str =
"
    varying vec2 out_coords;
    varying vec4 out_color;

    uniform sampler2D glyphs;

    float contour(float d, float w)
    {
        return smoothstep(0.5 - w, 0.5 + w, d);
    }

    float samp(vec2 uv, float w)
    {
        return contour(texture2D(glyphs, uv).a, w);
    }

    void main()
    {
        float alpha = 1.0;
        if(out_coords.s > -0.5)
        {
            vec2 uv = out_coords;
            float dist = texture2D(glyphs, uv).a;
            float width = fwidth(dist);
            alpha = contour(dist, width);
            float dscale = 0.354; //half of 1/sqrt2
            vec2 duv = dscale * (dFdx(uv) + dFdy(uv));
            vec4 box = vec4(uv - duv, uv + duv);
            float asum =
                samp(box.xy, width)
              + samp(box.zw, width)
              + samp(box.xw, width)
              + samp(box.zy, width);
            alpha = (alpha + 0.5 * asum) / 3.0;
        }
        gl_FragColor = alpha * out_color;
    }
";

#[repr(C, packed)]
struct Vertex
{
    pos: (f32, f32),
    coords: (f32, f32),
    color: (f32, f32, f32, f32)
}

impl AttributesReprCpacked for Vertex
{
    const ATTRIBUTES: &'static [(BufferType, &'static str)] =
    &[
        (BufferType::Float { size: 2 }, "in_pos"),
        (BufferType::Float { size: 2 }, "in_coords"),
        (BufferType::Float { size: 4 }, "in_color")
    ];
}

pub struct Binding
{
    pos: Vec2,
    events: Vec<UiEvent>,
    shader: Shader<Vertex>,
    tex_key: UniformKey<Texture<false>>,
    glyphs: Option<(u64, Texture<false>)>,
    vertices: VertexBuffer<Vertex>,
    indices: IndexBuffer,
    count: u32
}

impl Binding
{
    pub fn new(gl: &mut Gl) -> Self
    {
        #[cfg(not(target_arch = "wasm32"))]
        let shader = gl.new_shader(VERT, FRAG);
        #[cfg(target_arch = "wasm32")]
        let shader = gl.new_shader(VERT, &format!("    #extension GL_OES_standard_derivatives : require\n{FRAG}"));
        let tex_key = shader.get_key("glyphs");
        Self
        {
            pos: Vec2(0.0, 0.0),
            events: Vec::new(),
            shader,
            tex_key,
            glyphs: None,
            vertices: gl.new_vertex_buffer(0, BufferAccess::Dynamic),
            indices: gl.new_index_buffer(0, BufferAccess::Dynamic),
            count: 0
        }
    }

    pub fn event(&mut self, size: Vec2, event: &GlEvent) -> bool
    {
        let event = match event
        {
            GlEvent::Key { key, pressed } => convert_key(*key).map(|key| UiEvent::Key { key, pressed: *pressed }),
            GlEvent::Char(ch) => Some(UiEvent::Char(*ch)),
            GlEvent::Click { button, pressed } => Some(UiEvent::PointerClicked { pos: self.pos, button: convert_button(*button), pressed: *pressed }),
            GlEvent::Cursor { position } =>
            {
                self.pos = Vec2(position.0, size.1 - position.1);
                Some(UiEvent::PointerMoved { pos: self.pos, delta: Vec2(0.0, 0.0) })
            },
            GlEvent::CursorGone => Some(UiEvent::PointerGone),
            GlEvent::Scroll(GlScroll::Wheel(dx, dy) | GlScroll::Touch(dx, dy)) => Some(UiEvent::Scroll { dx: *dx, dy: *dy }),
            #[cfg(feature = "loading")]
            GlEvent::File(_) => None
        };
        match event
        {
            Some(event) =>
            {
                self.events.push(event);
                true
            },
            None => false
        }
    }

    pub fn events(&self) -> &[UiEvent]
    {
        &self.events
    }

    pub fn frame(&mut self, size: Vec2, gl: &mut Gl, frame: PaintFrame)
    {
        self.events.clear();

        let PaintFrame { new, vertices, indices, font_version, font_data } = frame;

        if self.glyphs.as_ref().map(|(version, _)| *version != font_version).unwrap_or(true)
        {
            if font_data.len() > 1 { unimplemented!("Multiple glyph textures ({}).", font_data.len()); }
            let config = TextureConfig { size: TEXTURE_SIZE, channel: TextureChannel::A, mipmap: false, wrap: TextureWrap::Clamp };
            let texture = gl.new_texture(&config, &font_data[0]);
            self.glyphs = Some((font_version, texture));
        }

        if new
        {
            let vertices: Vec<_> = vertices.iter().map(|vertex|
            {
                let pos = vertex.position.component_div(size / 2.0);
                let pos = Vec2(pos.0 - 1.0, 1.0 - pos.1);
                let coords = match vertex.tex_coords
                {
                    None => (-1.0, -1.0),
                    Some((s, t, l)) =>
                    {
                        if l > 0 { unreachable!(); }
                        (s, t)
                    }
                };
                Vertex
                {
                    pos: pos.into(),
                    coords,
                    color: vertex.color.to_normalized_srgb()
                }
            }).collect();
            if self.vertices.len() < vertices.len() as u32 { self.vertices = gl.new_vertex_buffer(vertices.len() as u32, BufferAccess::Dynamic); }
            if self.indices.len() < indices.len() as u32 { self.indices = gl.new_index_buffer(indices.len() as u32, BufferAccess::Dynamic); }
            self.vertices.data(0, &vertices);
            self.indices.data(0, &indices);
            self.count = indices.len() as u32;
        }
    }

    pub fn render(&mut self, rp: &mut RenderPass)
    {
        if let Some((_, glyphs)) = &self.glyphs
        {
            rp
                .pipeline(&self.shader, PipelineInfo { depth_test: false, alpha_blend: true, face_cull: true })
                .uniform_key(&self.tex_key, glyphs)
                .draw(Primitives::Triangles, &self.vertices, Some(&self.indices), 0, self.count);
        }
    }
}

pub fn convert_key(key: GlKey) -> Option<UiKey>
{
    match key
    {
        GlKey::Key1 => Some(UiKey::Key1),
        GlKey::Key2 => Some(UiKey::Key2),
        GlKey::Key3 => Some(UiKey::Key3),
        GlKey::Key4 => Some(UiKey::Key4),
        GlKey::Key5 => Some(UiKey::Key5),
        GlKey::Key6 => Some(UiKey::Key6),
        GlKey::Key7 => Some(UiKey::Key7),
        GlKey::Key8 => Some(UiKey::Key8),
        GlKey::Key9 => Some(UiKey::Key9),
        GlKey::Key0 => Some(UiKey::Key0),
        GlKey::A => Some(UiKey::A),
        GlKey::B => Some(UiKey::B),
        GlKey::C => Some(UiKey::C),
        GlKey::D => Some(UiKey::D),
        GlKey::E => Some(UiKey::E),
        GlKey::F => Some(UiKey::F),
        GlKey::G => Some(UiKey::G),
        GlKey::H => Some(UiKey::H),
        GlKey::I => Some(UiKey::I),
        GlKey::J => Some(UiKey::J),
        GlKey::K => Some(UiKey::K),
        GlKey::L => Some(UiKey::L),
        GlKey::M => Some(UiKey::M),
        GlKey::N => Some(UiKey::N),
        GlKey::O => Some(UiKey::O),
        GlKey::P => Some(UiKey::P),
        GlKey::Q => Some(UiKey::Q),
        GlKey::R => Some(UiKey::R),
        GlKey::S => Some(UiKey::S),
        GlKey::T => Some(UiKey::T),
        GlKey::U => Some(UiKey::U),
        GlKey::V => Some(UiKey::V),
        GlKey::W => Some(UiKey::W),
        GlKey::X => Some(UiKey::X),
        GlKey::Y => Some(UiKey::Y),
        GlKey::Z => Some(UiKey::Z),
        GlKey::Escape => Some(UiKey::Escape),
        GlKey::F1 => Some(UiKey::F1),
        GlKey::F2 => Some(UiKey::F2),
        GlKey::F3 => Some(UiKey::F3),
        GlKey::F4 => Some(UiKey::F4),
        GlKey::F5 => Some(UiKey::F5),
        GlKey::F6 => Some(UiKey::F6),
        GlKey::F7 => Some(UiKey::F7),
        GlKey::F8 => Some(UiKey::F8),
        GlKey::F9 => Some(UiKey::F9),
        GlKey::F10 => Some(UiKey::F10),
        GlKey::F11 => Some(UiKey::F11),
        GlKey::F12 => Some(UiKey::F12),
        GlKey::F13 => Some(UiKey::F13),
        GlKey::F14 => Some(UiKey::F14),
        GlKey::F15 => Some(UiKey::F15),
        GlKey::F16 => Some(UiKey::F16),
        GlKey::F17 => Some(UiKey::F17),
        GlKey::F18 => Some(UiKey::F18),
        GlKey::F19 => Some(UiKey::F19),
        GlKey::F20 => Some(UiKey::F20),
        GlKey::F21 => Some(UiKey::F21),
        GlKey::F22 => Some(UiKey::F22),
        GlKey::F23 => Some(UiKey::F23),
        GlKey::F24 => Some(UiKey::F24),
        GlKey::Snapshot => Some(UiKey::Snapshot),
        GlKey::Scroll => Some(UiKey::Scroll),
        GlKey::Pause => Some(UiKey::Pause),
        GlKey::Insert => Some(UiKey::Insert),
        GlKey::Home => Some(UiKey::Home),
        GlKey::Delete => Some(UiKey::Delete),
        GlKey::End => Some(UiKey::End),
        GlKey::PageDown => Some(UiKey::PageDown),
        GlKey::PageUp => Some(UiKey::PageUp),
        GlKey::Left => Some(UiKey::Left),
        GlKey::Up => Some(UiKey::Up),
        GlKey::Right => Some(UiKey::Right),
        GlKey::Down => Some(UiKey::Down),
        GlKey::Back => Some(UiKey::Back),
        GlKey::Return => Some(UiKey::Return),
        GlKey::Space => Some(UiKey::Space),
        GlKey::Numlock => Some(UiKey::Numlock),
        GlKey::Numpad0 => Some(UiKey::Numpad0),
        GlKey::Numpad1 => Some(UiKey::Numpad1),
        GlKey::Numpad2 => Some(UiKey::Numpad2),
        GlKey::Numpad3 => Some(UiKey::Numpad3),
        GlKey::Numpad4 => Some(UiKey::Numpad4),
        GlKey::Numpad5 => Some(UiKey::Numpad5),
        GlKey::Numpad6 => Some(UiKey::Numpad6),
        GlKey::Numpad7 => Some(UiKey::Numpad7),
        GlKey::Numpad8 => Some(UiKey::Numpad8),
        GlKey::Numpad9 => Some(UiKey::Numpad9),
        GlKey::NumpadAdd => Some(UiKey::NumpadAdd),
        GlKey::NumpadDivide => Some(UiKey::NumpadDivide),
        GlKey::NumpadDecimal => Some(UiKey::NumpadDecimal),
        GlKey::NumpadComma => Some(UiKey::NumpadComma),
        GlKey::NumpadEnter => Some(UiKey::NumpadEnter),
        GlKey::NumpadEquals => Some(UiKey::NumpadEquals),
        GlKey::NumpadMultiply => Some(UiKey::NumpadMultiply),
        GlKey::NumpadSubtract => Some(UiKey::NumpadSubtract),
        GlKey::LAlt => Some(UiKey::LAlt),
        GlKey::LControl => Some(UiKey::LControl),
        GlKey::LShift => Some(UiKey::LShift),
        GlKey::RAlt => Some(UiKey::RAlt),
        GlKey::RControl => Some(UiKey::RControl),
        GlKey::RShift => Some(UiKey::RShift),
        GlKey::Tab => Some(UiKey::Tab),
        _ => None
    }
}

pub fn convert_button(button: GlButton) -> UiButton
{
    match button
    {
        GlButton::Left => UiButton::Primary,
        GlButton::Right => UiButton::Secondary,
        GlButton::Middle => UiButton::Terciary,
        GlButton::Other(_) => UiButton::Terciary,
    }
}
