use std::rc::Rc;

use glam::*;
use rust_webgl2::{Graphics, Texture2DProps, FramebufferBinding, GlTexture2D, MagFilter, Viewport};

use super::{framebuffer::{Framebuffer, FramebufferKind, FramebufferAttachment}, framebuffer_blitter::FramebufferBlitter, texture_render::FramebufferAttachmentFormat};

pub struct TextureFramebuffer {
    render_framebuffer: Framebuffer,
    blit_framebuffer: Option<Framebuffer>,
}

impl TextureFramebuffer {
    pub fn new(
        graphics: &Graphics,
        size: UVec2,
        properties: Texture2DProps,
        texture_formats: &Vec<FramebufferAttachmentFormat>,
    ) -> Self {
        let require_renderbuffer = texture_formats
            .iter()
            .any(|format| format.only_renderbuffer);

        let mut render_framebuffer = Framebuffer::new(
            graphics,
            size,
            match require_renderbuffer {
                true => FramebufferKind::Renderbuffer { sample_count: 0 },
                false => FramebufferKind::Texture2D { properties },
            },
        );

        for format in texture_formats {
            render_framebuffer
                .create_color_texture(graphics, *format)
                .unwrap();
        }
        let blit_framebuffer = match require_renderbuffer {
            true => {
                let mut blit_framebuffer =
                    Framebuffer::new(graphics, size, FramebufferKind::Texture2D { properties });
                for format in texture_formats {
                    blit_framebuffer
                        .create_color_texture(graphics, *format)
                        .unwrap();
                }
                Some(blit_framebuffer)
            }
            false => None,
        };
        Self {
            render_framebuffer,
            blit_framebuffer,
        }
    }

    pub fn bind_render_buffer(&self, target: FramebufferBinding){
        self.render_framebuffer.bind(target);
    }

    pub fn get_texture_ref(&self, index: usize) -> Rc<GlTexture2D> {
        match &self.blit_framebuffer {
            Some(blit_framebuffer) => match &blit_framebuffer.color[index] {
                FramebufferAttachment::Renderbuffer(_) => panic!("Incorrect attachment type"),
                FramebufferAttachment::Texture2D(tx) => Rc::clone(tx),
            },
            None => match &self.render_framebuffer.color[index] {
                FramebufferAttachment::Renderbuffer(_) => panic!("Incorrect attachment type"),
                FramebufferAttachment::Texture2D(tx) => Rc::clone(tx),
            },
        }
    }

    pub fn blit_multi_attachment(&self, graphics: &Graphics, framebuffer_blitter: &FramebufferBlitter){
        let blit_framebuffer = self.blit_framebuffer.as_ref().expect("Blit framebuffer not created");
        framebuffer_blitter.blit_multi_attachment(
            graphics,
            &self.render_framebuffer,
            blit_framebuffer,
            MagFilter::LINEAR,
        );
    }

    pub fn blit_first_attachement(&self, graphics: &Graphics){
        let blit_framebuffer = self.blit_framebuffer.as_ref().expect("Blit framebuffer not created");
        let viewport = Viewport {
            position: UVec2::ZERO,
            size: self.render_framebuffer.size,
        };
        rust_webgl2::Framebuffer::blit_framebuffer(
            graphics,
            Some(&self.render_framebuffer.framebuffer),
            viewport,
            Some(&blit_framebuffer.framebuffer),
            viewport,
            true,
            false,
            false,
            MagFilter::LINEAR,
        )
    }

    pub fn has_blit_framebuffer(&self) -> bool {
        self.blit_framebuffer.is_some()
    }
}
