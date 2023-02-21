#[allow(dead_code)]

pub mod animation;
pub mod camera;
pub mod collision;
pub mod curves;
pub mod debug;
pub mod font_generation;
pub mod geometry;
pub mod gizmos;
pub mod js_file_fetcher;
pub mod math;
pub mod mesh;
pub mod random_collection;
pub mod renderer;
pub mod slotmap;
pub mod sdf_generation;
pub mod time;

const CAMERA_BINDING_NUMBER: u32 = 0;

pub fn set_camera_uniform_block_binding(program: &rust_webgl2::GlProgram) {
    program
        .set_uniform_block_binding_str("ViewMatrices", CAMERA_BINDING_NUMBER)
        .expect("Cannot set uniform block");
}

#[macro_export]
macro_rules! console_log_format {
	($($elem:expr),*) => {
		{
			use wasm_bindgen::JsValue;
			web_sys::console::log_1(&JsValue::from(&format!($($elem),*)))
		}
	};
}

pub trait Translatable {
    fn translate(&mut self, translate: glam::Vec3);
    fn set_position(&mut self, position: glam::Vec3);
    fn get_position(&self) -> glam::Vec3;
}

pub trait Orientable {
    fn rotate(&mut self, rotation: glam::Quat);
    fn set_orientation(&mut self, orientation: glam::Quat);
    fn get_orientation(&self) -> glam::Quat;
}

pub trait Scalable {
    fn scale(&mut self, scale: glam::Vec3);
    fn set_scale(&mut self, scale: glam::Vec3);
    fn get_scale(&self) -> glam::Vec3;
}
