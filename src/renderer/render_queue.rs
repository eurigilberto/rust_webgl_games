use std::collections::HashMap;

use rust_webgl2::Graphics;

use super::{Renderer, RenderState};

pub type RenderRequest = Box<dyn Fn(&Graphics, &RenderState) -> ()>;
pub struct RenderRequestLayer {
    layers: Vec<RenderRequest>,
}
impl RenderRequestLayer {
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
        }
    }

    pub fn insert_render_request(&mut self, request: RenderRequest) {
        self.layers.push(request);
    }

    pub fn clear_requests(&mut self) {
        self.layers.clear();
    }

    pub fn execute_requests(&self, graphics: &Graphics, render_state: &RenderState) {
        for layer in self.layers.iter() {
            layer(graphics, render_state);
        }
    }
}

pub struct RenderQueue {
    pub queues: Vec<RenderRequestLayer>,
}

impl RenderQueue {
    pub fn new(requestLayerCount: usize) -> Self {
        Self {
            queues: (0..requestLayerCount)
                .map(|_| RenderRequestLayer::new())
                .collect(),
        }
    }

    pub fn clear_requests(&mut self) {
        self.queues.iter_mut().for_each(|queue| queue.clear_requests());
    }
}

impl Renderer{
	pub fn insert_opaque_render_request(&self, request: RenderRequest, layer: usize) {
        self.render_queue.borrow_mut().queues[layer].insert_render_request(request);
    }
}