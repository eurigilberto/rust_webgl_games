use std::cell::RefCell;

use glam::*;
use rust_webgl2::{
    DrawCapabilities, Graphics, MagFilter, MinFilter, Texture2DProps, TextureInternalFormat,
    TextureWrap, Viewport, RGBA,
};

mod render_queue;
pub mod render_texture_quad;
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
    pub render_ops: Vec<RenderOp>,
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

#[derive(Clone, Copy)]
pub enum RenderOp{
    ExecuteRequests{
        layer: usize,
    },
    BlitToColor,
    BlitToCanvas{
        canvas_viewport: Viewport
    }
}

impl Renderer {
    pub fn clear_requests(&self) {
        self.render_queue.borrow_mut().clear_requests();
    }

    pub fn resize(&mut self, size: UVec2) {
        self.graphics.resize(size);
    }

    pub fn new(
        clear_state: ClearState,
        canvas: HtmlCanvasElement,
        context: WebGl2RenderingContext,
        sample_count: u32,
        request_layer_count: usize
    ) -> Self {
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
            render_queue: RefCell::new(RenderQueue::new(request_layer_count)),
            render_state,
            sample_count,
            render_ops: Vec::new(),
        }
    }

    pub fn finish(&self) {
        self.graphics.finish();
    }

    pub fn get_graphics(&self) -> &Graphics {
        &self.graphics
    }

    pub fn create_main_render_textures(&mut self, size: UVec2) -> Result<(), ()> {
        let mut render_fb = Framebuffer::new(
            &self.graphics,
            size,
            FramebufferKind::Renderbuffer {
                sample_count: self.sample_count,
            },
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
    
    fn render_buffer_framebuffer_ref(&self) -> &rust_webgl2::Framebuffer {
        &self.render_state.render_buffers.as_ref().unwrap().framebuffer
    }
    fn render_buffer_copy_framebuffer_ref(&self) -> &rust_webgl2::Framebuffer {
        &self.render_state.render_buffers_copy.as_ref().unwrap().framebuffer
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
        let render_framebuffer = self.render_buffer_framebuffer_ref();
        let color_framebuffer = self.render_buffer_copy_framebuffer_ref();
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

    fn create_main_render_texture_if_required(&mut self, texture_size: UVec2) -> Result<(), ()> {
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

    pub fn render(&mut self, render_texture_size: UVec2) -> Result<(), ()> {
        self.create_main_render_texture_if_required(render_texture_size)?;

        self.render_state.set_main_framebuffer(&self.graphics);

        self.graphics.clear_current_framebuffer(
            self.render_state.clear_state.color,
            self.render_state.clear_state.depth,
            self.render_state.clear_state.stencil,
        );

        for op_index in 0..self.render_ops.len(){
            let op = self.render_ops[op_index];
            match op{
                RenderOp::ExecuteRequests{layer} => {
                    self.render_queue.borrow_mut().queues[layer].execute_requests(&self.graphics, &self.render_state)
                },
                RenderOp::BlitToColor => {
                    self.blit_to_color_buffer()?;
                },
                RenderOp::BlitToCanvas{canvas_viewport} => {
                    self.blit_to_canvas(canvas_viewport);
                }
            }
        }
        Ok(())
    }

    pub fn resize_canvas_if_required(&mut self, screen_size: UVec2) {
        if self.graphics.get_canvas_size() != screen_size{
            self.resize(screen_size);
        }
    }

    pub fn blit_to_canvas(&self, canvas_viewport: Viewport) {
        match &self.render_state.render_buffers {
            Some(texture_props) => {
                let src_viewport = Viewport {
                    position: glam::UVec2::ZERO,
                    size: texture_props.size,
                };
                rust_webgl2::Framebuffer::blit_framebuffer(
                    self.get_graphics(),
                    Some(
                        &self
                            .render_state
                            .render_buffers_copy
                            .as_ref()
                            .expect("Should not be none if the render buffers are not none")
                            .framebuffer,
                    ),
                    src_viewport,
                    None,
                    canvas_viewport,
                    true,
                    false,
                    false,
                    MagFilter::NEAREST,
                )
            }
            None => { /* NO OP */ }
        }
    }
}
