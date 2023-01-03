use std::{cell::RefCell};

use glam::*;
use rust_webgl2::{
    DrawCapabilities, Graphics, MagFilter, MinFilter,
    Texture2DProps, TextureInternalFormat, TextureWrap, Viewport, RGBA,
};

pub mod render_texture_quad;
mod render_queue;
pub mod texture_render;
use render_queue::*;
pub mod framebuffer;
use framebuffer::*;
mod framebuffer_blitter;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};


pub struct RenderState {
    pub clear_state: ClearState,
    pub render_buffers: Option<Framebuffer>,
    pub render_buffers_copy: Option<Framebuffer>,
}

impl RenderState {
    pub fn set_main_framebuffer(&self, graphics: &Graphics) {
        if let Some(framebuffer) = &self.render_buffers {
            graphics.set_viewport(IVec2::ZERO, framebuffer.size);
            framebuffer.bind(rust_webgl2::FramebufferBinding::DRAW_FRAMEBUFFER);
        }
    }
}

#[derive(Clone, Copy)]
pub struct ClearState {
    pub color: Option<RGBA>,
    pub depth: Option<f32>,
    pub stencil: Option<u32>,
}

const RENDER_TEXTURE_PROPS: Texture2DProps = Texture2DProps {
    wrap_x: TextureWrap::CLAMP_TO_EDGE,
    wrap_y: TextureWrap::CLAMP_TO_EDGE,
    mag_filter: MagFilter::LINEAR,
    min_filter: MinFilter::LINEAR,
    base_level: 0,
    max_level: 1,
    min_max_lod: (0.0, 0.0),
};

pub struct Renderer {
    sample_count: u32,
    graphics: Graphics,
    pub render_state: RenderState,
    render_queue: RefCell<RenderQueue>,
}

pub const BLIT_CAPABILITIES: DrawCapabilities = DrawCapabilities {
    blend_state: None,
    cull_face: None,
    depth_test: None,
    stencil_test: None,
    scissor_test: None,
    color_draw_mask: (true, true, true, true),
    depth_draw_mask: true,
};

impl Renderer {
    pub fn clear_requests(&self) {
        self.render_queue.borrow_mut().clear_requests();
    }

    pub fn resize(&mut self, size: UVec2){
        self.graphics.resize(size);
    }

    pub fn new(clear_state: ClearState, canvas: HtmlCanvasElement, context: WebGl2RenderingContext, sample_count: u32) -> Self {
        let graphics = {
            context
                .get_extension("EXT_color_buffer_float")
                .expect("Color buffer floats cannot be used");
            Graphics::new(context, canvas).expect("Cannot create graphics object")
        };

        let render_state = RenderState {
            clear_state,
            render_buffers: None,
            render_buffers_copy: None,
        };

        Self {
            graphics,
            render_queue: RefCell::new(RenderQueue::new()),
            render_state,
            sample_count
        }
    }

    pub fn get_graphics(&self) -> &Graphics {
        &self.graphics
    }

    pub fn create_main_render_textures(&mut self, size: UVec2) -> Result<(), ()> {
        let mut render_fb = Framebuffer::new(
            &self.graphics,
            size,
            FramebufferKind::Renderbuffer { sample_count: self.sample_count },
        );
        let mut color_fb = Framebuffer::new(
            &self.graphics,
            size,
            FramebufferKind::Texture2D {
                properties: RENDER_TEXTURE_PROPS,
            },
        );

        render_fb.create_color_texture(&self.graphics, TextureInternalFormat::RGBA8)?;
        render_fb.create_depth_texture(&self.graphics, TextureInternalFormat::DEPTH24_STENCIL8)?;

        color_fb.create_color_texture(&self.graphics, TextureInternalFormat::RGBA8)?;

        self.render_state.render_buffers = Some(render_fb);
        self.render_state.render_buffers_copy = Some(color_fb);

        Ok(())
    }

    fn blit_to_color_buffer(&mut self) -> Result<(), ()> {
        if self.render_state.render_buffers.is_none()
            || self.render_state.render_buffers_copy.is_none()
        {
            return Err(());
        }

        let size = self.render_state.render_buffers.as_ref().unwrap().size;
        let viewport = Viewport {
            position: UVec2::ZERO,
            size,
        };
        BLIT_CAPABILITIES.set_capabilities(&self.graphics);
        let render_framebuffer = &self
            .render_state
            .render_buffers
            .as_ref()
            .unwrap()
            .framebuffer;
        let color_framebuffer = &self
            .render_state
            .render_buffers_copy
            .as_ref()
            .unwrap()
            .framebuffer;
        rust_webgl2::Framebuffer::blit_framebuffer(
            &self.graphics,
            Some(render_framebuffer),
            viewport,
            Some(color_framebuffer),
            viewport,
            true,
            false,
            false,
            MagFilter::NEAREST,
        );
        rust_webgl2::Framebuffer::bind(
            render_framebuffer,
            rust_webgl2::FramebufferBinding::DRAW_FRAMEBUFFER,
        );
        Ok(())
    }

    fn create_main_render_texture_if_required(&mut self, texture_size: UVec2)->Result<(),()> {
        match &mut self.render_state.render_buffers {
            Some(texture_props) => {
                if texture_props.size != texture_size {
                    self.create_main_render_textures(texture_size)?;
                }
                Ok(())
            }
            None => {
                self.create_main_render_textures(texture_size)?;
                Ok(())
            }
        }
    }

    pub fn render(&mut self, texture_size: UVec2)->Result<(),()> {
        self.create_main_render_texture_if_required(texture_size)?;

        self.render_state.set_main_framebuffer(&self.graphics);

        self.graphics.clear_current_framebuffer(
            self.render_state.clear_state.color,
            self.render_state.clear_state.depth,
            self.render_state.clear_state.stencil,
        );

        self.execute_opaque_requests();
        //self.blit_to_color_buffer()?;
        self.execute_after_opaque_requests();
        self.execute_transparent_requests();
        //self.blit_to_color_buffer()?;
        self.execute_after_transparent_requests();
        self.blit_to_color_buffer()?;
        Ok(())
    }

    fn execute_opaque_requests(&self) {
        self.render_queue
            .borrow_mut()
            .opaque_queue
            .execute_requests(&self.graphics, &self.render_state);
    }
    fn execute_after_opaque_requests(&self) {
        self.render_queue
            .borrow_mut()
            .after_opaque_render
            .execute_requests(&self.graphics, &self.render_state);
    }
    fn execute_transparent_requests(&self) {
        self.render_queue
            .borrow_mut()
            .transparent_queue
            .execute_requests(&self.graphics, &self.render_state);
    }
    fn execute_after_transparent_requests(&self) {
        self.render_queue
            .borrow_mut()
            .after_transparent_render
            .execute_requests(&self.graphics, &self.render_state);
    }
}
