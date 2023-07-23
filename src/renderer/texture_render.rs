use glam::*;
use rust_webgl2::{
    CullMode, DrawCapabilities, FunctionDefinition, GlMaterial, GlTexture2D, GlUniform, Graphics,
    IntoGlUniform, MagFilter, PrimitiveType, ShaderSource, ShaderStage, ShaderUniform,
    Texture2DProps, TextureInternalFormat, UniformCollection, UniformIndex, Viewport,
    WebGLDataType,
};
use std::rc::Rc;

use super::{
    framebuffer::*, framebuffer_blitter::FramebufferBlitter,
    texture_framebuffer::TextureFramebuffer,
};

fn texture_shader_vertex_stage() -> ShaderStage {
    ShaderStage {
        import_fn: vec![],
        main_fn: r#"
vec2 vert_position[4] = vec2[4](
vec2( 1.0, -1.0),
vec2(-1.0, -1.0),
vec2( 1.0,  1.0),
vec2(-1.0,  1.0)
);
int v_id = gl_VertexID;
gl_Position = vec4(vert_position[v_id], 0.0, 1.0);
"#
        .into(),
        attributes: vec![],
        uniform_collection: UniformCollection::new(),
    }
}

pub struct TextureShaderSource {
    pub name: String,
    pub imported_functions: Vec<FunctionDefinition>,
    pub fragment_shader: ShaderStage,
}

impl TextureShaderSource {
    pub fn get_texture_shader_source(self) -> ShaderSource {
        ShaderSource {
            name: self.name,
            varyings: vec![],
            common_uniforms: UniformCollection {
                uniforms: vec![ShaderUniform {
                    array_length: None,
                    kind: WebGLDataType::Vec2,
                    name: "texture_size".into(),
                }],
                uniform_blocks: vec![],
            },
            imported_functions: self.imported_functions,
            vertex_shader: texture_shader_vertex_stage(),
            fragment_shader: self.fragment_shader,
            local_import: None
        }
    }
}

pub struct DepthRenderable(TextureInternalFormat);
impl DepthRenderable {
    pub const DEPTH_COMPONENT16: Self = DepthRenderable(TextureInternalFormat::DEPTH_COMPONENT16);
    pub const DEPTH_COMPONENT24: Self = DepthRenderable(TextureInternalFormat::DEPTH_COMPONENT24);
    pub const DEPTH_COMPONENT32F: Self = DepthRenderable(TextureInternalFormat::DEPTH_COMPONENT32F);
    pub const DEPTH24_STENCIL8: Self = DepthRenderable(TextureInternalFormat::DEPTH24_STENCIL8);
}

pub struct ColorRenderable(TextureInternalFormat);
impl ColorRenderable {
    pub const R8: Self = ColorRenderable(TextureInternalFormat::R8);
    pub const RG8: Self = ColorRenderable(TextureInternalFormat::RG8);
    pub const RGB8: Self = ColorRenderable(TextureInternalFormat::RGB8);
    pub const RGB565: Self = ColorRenderable(TextureInternalFormat::RGB565);
    pub const RGBA4: Self = ColorRenderable(TextureInternalFormat::RGBA4);
    pub const RGB5_A1: Self = ColorRenderable(TextureInternalFormat::RGB5_A1);
    pub const RGBA8: Self = ColorRenderable(TextureInternalFormat::RGBA8);

    pub const RGB10_A2: Self = ColorRenderable(TextureInternalFormat::RGB10_A2);
    pub const RGB10_A2UI: Self = ColorRenderable(TextureInternalFormat::RGB10_A2UI);
    pub const SRGB8_ALPHA8: Self = ColorRenderable(TextureInternalFormat::SRGB8_ALPHA8);

    pub const R8I: Self = ColorRenderable(TextureInternalFormat::R8I);
    pub const R8UI: Self = ColorRenderable(TextureInternalFormat::R8UI);
    pub const R16I: Self = ColorRenderable(TextureInternalFormat::R16I);
    pub const R16UI: Self = ColorRenderable(TextureInternalFormat::R16UI);
    pub const R32I: Self = ColorRenderable(TextureInternalFormat::R32I);
    pub const R32UI: Self = ColorRenderable(TextureInternalFormat::R32UI);
    pub const RG8I: Self = ColorRenderable(TextureInternalFormat::RG8I);
    pub const RG8UI: Self = ColorRenderable(TextureInternalFormat::RG8UI);
    pub const RG16I: Self = ColorRenderable(TextureInternalFormat::RG16I);
    pub const RG16UI: Self = ColorRenderable(TextureInternalFormat::RG16UI);
    pub const RG32I: Self = ColorRenderable(TextureInternalFormat::RG32I);
    pub const RG32UI: Self = ColorRenderable(TextureInternalFormat::RG32UI);
    pub const RGBA8I: Self = ColorRenderable(TextureInternalFormat::RGBA8I);
    pub const RGBA8UI: Self = ColorRenderable(TextureInternalFormat::RGBA8UI);
    pub const RGBA16I: Self = ColorRenderable(TextureInternalFormat::RGBA16I);
    pub const RGBA16UI: Self = ColorRenderable(TextureInternalFormat::RGBA16UI);
    pub const RGBA32I: Self = ColorRenderable(TextureInternalFormat::RGBA32I);
    pub const RGBA32UI: Self = ColorRenderable(TextureInternalFormat::RGBA32UI);
}

pub struct RBColorRenderable(TextureInternalFormat);
impl RBColorRenderable {
    pub const R16F: Self = RBColorRenderable(TextureInternalFormat::R16F);
    pub const RG16F: Self = RBColorRenderable(TextureInternalFormat::RG16F);
    pub const RGBA16F: Self = RBColorRenderable(TextureInternalFormat::RGBA16F);
    pub const R32F: Self = RBColorRenderable(TextureInternalFormat::R32F);
    pub const RG32F: Self = RBColorRenderable(TextureInternalFormat::RG32F);
    pub const RGBA32F: Self = RBColorRenderable(TextureInternalFormat::RGBA32F);
    pub const R11F_G11F_B10F: Self = RBColorRenderable(TextureInternalFormat::R11F_G11F_B10F);
}

#[derive(Clone, Copy)]
pub struct FramebufferAttachmentFormat{
    pub format: TextureInternalFormat,
    pub only_renderbuffer: bool,
    pub is_depth: bool,
}

impl Into<FramebufferAttachmentFormat> for DepthRenderable{
    fn into(self) -> FramebufferAttachmentFormat {
        FramebufferAttachmentFormat {
            format: self.0,
            is_depth: true,
            only_renderbuffer: false,
        }
    }
}

impl Into<FramebufferAttachmentFormat> for ColorRenderable{
    fn into(self) -> FramebufferAttachmentFormat {
        FramebufferAttachmentFormat {
            format: self.0,
            is_depth: false,
            only_renderbuffer: false,
        }
    }
}

impl Into<FramebufferAttachmentFormat> for RBColorRenderable{
    fn into(self) -> FramebufferAttachmentFormat {
        FramebufferAttachmentFormat {
            format: self.0,
            is_depth: false,
            only_renderbuffer: true,
        }
    }
}

pub struct TextureShaderRender {
    pub material: GlMaterial,
    size: UVec2,
    pub framebuffer: TextureFramebuffer,
    framebuffer_blitter: Option<FramebufferBlitter>,
}

impl TextureShaderRender {
    pub fn new(
        graphics: &Graphics,
        properties: Texture2DProps,
        size: UVec2,
        texture_formats: Vec<FramebufferAttachmentFormat>,
        shader: TextureShaderSource,
        name: Option<String>
    ) -> Result<Self, ()> {
        let framebuffer = TextureFramebuffer::new(graphics, size, properties, &texture_formats, name);
        let framebuffer_blitter = if framebuffer.has_blit_framebuffer() && texture_formats.len() > 1 {
            Some(FramebufferBlitter::new(graphics))
        } else {
            None
        };

        let mut material = GlMaterial::with_source(
            graphics,
            vec![DrawCapabilities {
                cull_face: Some(CullMode::FRONT),
                ..Default::default()
            }],
            &shader.get_texture_shader_source(),
        )
        .expect("Material creation error");

        let uniform_size = size.as_vec2().uniform();
        match material
            .program
            .insert_uniform("texture_size", uniform_size)
        {
            Ok(_) => {}
            Err(_) => { /* texture size might not be in use */ }
        }
        Ok(Self {
            framebuffer,
            material,
            size,
            framebuffer_blitter,
        })
    }

    pub fn insert_uniforms(
        &mut self,
        uniforms: Vec<(String, GlUniform)>,
    ) -> Result<Vec<UniformIndex>, String> {
        let mut uniform_indices = Vec::new();
        for (name, uniform) in uniforms.into_iter() {
            match self.material.program.insert_uniform(&name, uniform) {
                Ok(u_index) => {
                    uniform_indices.push(u_index);
                }
                Err(_) => return Err(name),
            }
        }
        Ok(uniform_indices)
    }

    pub fn render_texture(&mut self, graphics: &Graphics) {
        graphics.set_viewport(IVec2::ZERO, self.size);
        graphics.set_depth_range(0.0, 2.0);

        self.material.set_capabilities(graphics, 0);
        self.material.push_texture_samplers(graphics);
        let mut current_program = self.material.program.use_program();
        self.framebuffer
            .bind_render_buffer(rust_webgl2::FramebufferBinding::DRAW_FRAMEBUFFER);
        current_program.push_all_uniforms();
        current_program.draw_arrays(PrimitiveType::TRIANGLE_STRIP, 0, 4);

        if let Some(framebuffer_blitter) = &self.framebuffer_blitter {
            self.framebuffer
                .blit_multi_attachment(graphics, framebuffer_blitter);
        } else if self.framebuffer.has_blit_framebuffer(){
            self.framebuffer.blit_first_attachement(graphics);
        }
    }
}
