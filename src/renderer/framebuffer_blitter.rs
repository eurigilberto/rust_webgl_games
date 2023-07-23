#![allow(dead_code)]
use glam::uvec2;
use rust_webgl2::{Graphics, Viewport, MagFilter};

use super::framebuffer::{Framebuffer, FramebufferAttachment};

const FIRST_ATTACHMENT: rust_webgl2::FramebufferAttachment =
    rust_webgl2::FramebufferAttachment::Color(0);

pub struct FramebufferBlitter {
    source_framebuffer: rust_webgl2::Framebuffer,
    destination_framebuffer: rust_webgl2::Framebuffer,
}

impl FramebufferBlitter {
    pub fn new(graphics: &Graphics) -> Self {
        Self {
            source_framebuffer: rust_webgl2::Framebuffer::new(graphics).unwrap(),
            destination_framebuffer: rust_webgl2::Framebuffer::new(graphics).unwrap(),
        }
    }

    fn attach_color_binding(
        framebuffer: &rust_webgl2::Framebuffer,
        attachment: &FramebufferAttachment,
    ) {
        match attachment {
            FramebufferAttachment::Renderbuffer(renderbuffer) => {
                framebuffer.set_attachment_renderbuffer(FIRST_ATTACHMENT, Some(renderbuffer));
            }
            FramebufferAttachment::Texture2D(texture) => {
                framebuffer.set_attachment_texture2d(FIRST_ATTACHMENT, Some(texture));
            }
        }
    }

    pub fn blit_multi_attachment(
        &self,
        graphics: &Graphics,
        source: &Framebuffer,
        destination: &Framebuffer,
        filter: MagFilter
    ) {
        for (src_attachment, dst_attachment) in source.color.iter().zip(destination.color.iter()) {
            Self::attach_color_binding(&self.source_framebuffer, src_attachment);
            Self::attach_color_binding(&self.destination_framebuffer, dst_attachment);

            let src_viewport = Viewport {
                position: uvec2(0, 0),
                size: source.size,
            };

            let dst_viewport = Viewport {
                position: uvec2(0, 0),
                size: destination.size,
            };

            rust_webgl2::Framebuffer::blit_framebuffer(
                graphics,
                Some(&self.source_framebuffer),
                src_viewport,
                Some(&self.destination_framebuffer),
                dst_viewport,
                true,
                false,
                false,
                filter,
            );
        }
    }
}
