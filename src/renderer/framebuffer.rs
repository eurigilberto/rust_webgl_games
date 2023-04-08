use std::rc::Rc;

use glam::*;
use rust_webgl2::{GlTexture2D, Graphics, Renderbuffer, Texture2DProps, TextureInternalFormat, FramebufferBinding};

use crate::console_log_format;

use super::texture_render::{ColorRenderable, RBColorRenderable, DepthRenderable, FramebufferAttachmentFormat};

pub fn is_depth(format: TextureInternalFormat) -> bool {
    match format {
        TextureInternalFormat::DEPTH24_STENCIL8
        | TextureInternalFormat::DEPTH_COMPONENT16
        | TextureInternalFormat::DEPTH_COMPONENT24
        | TextureInternalFormat::DEPTH_COMPONENT32F => true,
        _ => false,
    }
}



#[derive(Clone, Copy)]
pub struct FramebufferAttachmentProperties {
    pub size: UVec2,
    pub kind: FramebufferKind,
    pub format: TextureInternalFormat,
}

#[derive(Clone, Copy)]
pub enum FramebufferKind {
    Texture2D { properties: Texture2DProps},
    Renderbuffer { sample_count: u32 },
}

fn create_and_bind_attachment(
    graphics: &Graphics,
    framebuffer_properties: FramebufferAttachmentProperties,
    framebuffer: &rust_webgl2::Framebuffer,
    attachment: rust_webgl2::FramebufferAttachment,
    name: Option<String>
) -> Result<FramebufferAttachment, ()> {
    match framebuffer_properties.kind {
        FramebufferKind::Texture2D { properties } => {
            let texture = GlTexture2D::new(graphics, properties, framebuffer_properties.size, framebuffer_properties.format, None, name)?;
            framebuffer.set_attachment_texture2d(attachment, Some(&texture));
            Ok(FramebufferAttachment::Texture2D(Rc::new(texture)))
        }
        FramebufferKind::Renderbuffer { sample_count } => {
            let renderbuffer = Renderbuffer::new(graphics, name, sample_count, framebuffer_properties.size, framebuffer_properties.format)?;
            framebuffer.set_attachment_renderbuffer(attachment, Some(&renderbuffer));
            Ok(FramebufferAttachment::Renderbuffer(Rc::new(renderbuffer)))
        }
    }
}

pub enum FramebufferAttachment {
    Renderbuffer(Rc<Renderbuffer>),
    Texture2D(Rc<GlTexture2D>),
}

pub struct Framebuffer {
    pub size: UVec2,
    pub kind: FramebufferKind,
    pub framebuffer: rust_webgl2::Framebuffer,
    pub color: Vec<FramebufferAttachment>,
    pub depth: Option<FramebufferAttachment>,
}

#[derive(Debug, Clone, Copy)]
pub enum CreateAttachmentError{
    FormatOnlyAvailableOnRenderbuffer,
    AttachmentCreationFailed,
    ExpectedDepthTextureFormat,
    ExpectedColorTextureFormat,
}

impl Framebuffer {
    pub fn new(graphics: &Graphics, size: UVec2, kind: FramebufferKind) -> Self {
        let framebuffer =
            rust_webgl2::Framebuffer::new(graphics).expect("Could not create Framebuffer");
        Self {
            size,
            kind,
            framebuffer: framebuffer,
            color: Vec::new(),
            depth: None,
        }
    }

    pub fn bind(&self, target: FramebufferBinding){
        self.framebuffer.bind(target);
        if target == FramebufferBinding::DRAW_FRAMEBUFFER{
            let mut buffers = Vec::new();
            for i in 0..self.color.len(){
                buffers.push(i as u32);
            }
            self.framebuffer.set_draw_buffers(buffers);
        }
    }
    
    pub fn create_color_texture(
        &mut self,
        graphics: &Graphics,
        format: FramebufferAttachmentFormat,
        name: Option<String>,
    ) -> Result<(), CreateAttachmentError> {
        if format.is_depth {
            return Err(CreateAttachmentError::ExpectedColorTextureFormat);
        }
        if format.only_renderbuffer {
            match self.kind {
                FramebufferKind::Texture2D { .. } => {
                    return Err(CreateAttachmentError::FormatOnlyAvailableOnRenderbuffer)
                }
                _ => {}
            }
        }

        let attachment = rust_webgl2::FramebufferAttachment::Color(self.color.len() as u32);
        let framebuffer_props = FramebufferAttachmentProperties {
            size: self.size,
            kind: self.kind,
            format: format.format,
        };

        let buffer = match create_and_bind_attachment(
            graphics,
            framebuffer_props,
            &self.framebuffer,
            attachment,
            name
        ){
            Ok(buffer) => buffer,
            Err(_) => return Err(CreateAttachmentError::AttachmentCreationFailed),
        };
        
        self.color.push(buffer);
        Ok(())
    }

    pub fn create_depth_texture(
        &mut self,
        graphics: &Graphics,
        format: FramebufferAttachmentFormat,
        name: Option<String>,
    ) -> Result<(), CreateAttachmentError> {
        if !format.is_depth {
            return Err(CreateAttachmentError::ExpectedDepthTextureFormat)
        }

        let attachment = match format.format {
            TextureInternalFormat::DEPTH24_STENCIL8 => {
                rust_webgl2::FramebufferAttachment::DepthStencil
            }
            _ => rust_webgl2::FramebufferAttachment::Depth,
        };

        let framebuffer_props = FramebufferAttachmentProperties {
            size: self.size,
            kind: self.kind,
            format: format.format,
        };

        let buffer = match create_and_bind_attachment(
            graphics,
            framebuffer_props,
            &self.framebuffer,
            attachment,
            name
        ){
            Ok(buffer) => buffer,
            Err(_) => return Err(CreateAttachmentError::AttachmentCreationFailed),
        };

        self.depth = Some(buffer);
        Ok(())
    }
}

impl Drop for Framebuffer{
    fn drop(&mut self) {
        //console_log_format!("Dropped framebuffer with | color count : {} | depth: {}", self.color.len(), self.depth.is_some())
    }
}
