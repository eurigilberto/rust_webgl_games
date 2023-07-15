use rust_webgl2::Graphics;

use super::{Renderer, RenderState};

pub type RenderRequest = Box<dyn Fn(&Graphics, &RenderState) -> ()>;
pub struct RenderRequestLayer {
    requests: Vec<RenderRequest>,
}
impl RenderRequestLayer {
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
        }
    }

    pub fn insert_render_request(&mut self, request: RenderRequest) {
        self.requests.push(request);
    }

    pub fn clear_requests(&mut self) {
        self.requests.clear();
    }

    pub fn execute_requests(&self, graphics: &Graphics, render_state: &RenderState) {
        for request in self.requests.iter() {
            request(graphics, render_state);
        }
    }
}

pub struct RenderQueue {
    pub queues: Vec<RenderRequestLayer>,
}

impl RenderQueue {
    pub fn new(request_layer_count: usize) -> Self {
        Self {
            queues: (0..request_layer_count)
                .map(|_| RenderRequestLayer::new())
                .collect(),
        }
    }

    pub fn clear_requests(&mut self) {
        self.queues.iter_mut().for_each(|queue| queue.clear_requests());
    }
}

impl Renderer{
	pub fn insert_render_request(&self, request: RenderRequest, layer: usize) {
        self.render_queue.borrow_mut().queues[layer].insert_render_request(request);
    }
}