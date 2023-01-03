use std::collections::HashMap;

use rust_webgl2::Graphics;

use super::{Renderer, RenderState};

pub type RenderRequest = Box<dyn Fn(&Graphics, &RenderState) -> ()>;
pub struct RenderRequestLayers {
    layers: HashMap<usize, Vec<RenderRequest>>,
}
impl RenderRequestLayers {
    pub fn new() -> Self {
        Self {
            layers: HashMap::new(),
        }
    }

    pub fn insert_render_request(&mut self, request: RenderRequest, layer: usize) {
        if !self.layers.contains_key(&layer) {
            self.layers.insert(layer, Vec::new());
        }
        let layer = self.layers.get_mut(&layer).unwrap();
        layer.push(request);
    }

    pub fn clear_requests(&mut self) {
        for (_, layer) in self.layers.iter_mut() {
            layer.clear();
        }
    }

    pub fn execute_requests(&self, graphics: &Graphics, render_state: &RenderState) {
        for (_, layer) in self.layers.iter() {
            for request in layer.iter() {
                request(graphics, render_state);
            }
        }
    }
}

pub struct RenderQueue {
    pub opaque_queue: RenderRequestLayers,
    pub after_opaque_render: RenderRequestLayers,
    pub transparent_queue: RenderRequestLayers,
    pub after_transparent_render: RenderRequestLayers,
}

impl RenderQueue {
    pub fn new() -> Self {
        Self {
            opaque_queue: RenderRequestLayers::new(),
            after_opaque_render: RenderRequestLayers::new(),
            transparent_queue: RenderRequestLayers::new(),
            after_transparent_render: RenderRequestLayers::new(),
        }
    }

    pub fn clear_requests(&mut self) {
        self.opaque_queue.clear_requests();
        self.after_opaque_render.clear_requests();
        self.transparent_queue.clear_requests();
        self.after_transparent_render.clear_requests();
    }
}

impl Renderer{
	pub fn insert_opaque_render_request(&self, request: RenderRequest, layer: usize) {
        self.render_queue
            .borrow_mut()
            .opaque_queue
            .insert_render_request(request, layer);
    }
    pub fn insert_after_opaque_render_request(&self, request: RenderRequest, layer: usize) {
        self.render_queue
            .borrow_mut()
            .after_opaque_render
            .insert_render_request(request, layer);
    }
    pub fn insert_transparent_render_request(&self, request: RenderRequest, layer: usize) {
        self.render_queue
            .borrow_mut()
            .transparent_queue
            .insert_render_request(request, layer);
    }
    pub fn insert_after_transparent_render_request(&self, request: RenderRequest, layer: usize) {
        self.render_queue
            .borrow_mut()
            .after_transparent_render
            .insert_render_request(request, layer);
    }
}