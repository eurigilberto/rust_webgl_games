#![allow(dead_code)]

use glam::*;

pub fn f32_lerp(a: f32,b: f32, t: f32)->f32{
	a + t * (b - a)
}

pub fn f32_inverse_lerp(a: f32, b: f32, t: f32)->f32{
	(t - a)/(b - a)
}

pub fn cross_vec2(a: Vec2, b: Vec2)->f32{
	a.x * b.y - a.y * b.x
}

pub fn to_vec4(value: Vec3, w: f32)->Vec4{
	vec4(value.x, value.y, value.z, w)
}

pub fn vec4_from_vec2(xy: Vec2, zw: Vec2) -> Vec4{
	vec4(xy.x, xy.y, zw.x, zw.y)
}

#[derive(Default, Clone,Copy)]
pub struct MinMaxF32{
	pub min: f32,
	pub max: f32
}

impl MinMaxF32{
	pub fn new(min: f32, max: f32)->Self{
		Self { min, max }
	}
	pub fn lerp(&self, param: f32)->f32{
		f32_lerp(self.min, self.max, param)
	}
	pub fn random(&self)->f32{
		let random = rand::random();
		self.lerp(random)
	}
}

#[derive(Default, Clone,Copy)]
pub struct MinMaxVec2{
	pub min: Vec2,
	pub max: Vec2
}

impl MinMaxVec2{
	pub fn new(min: Vec2, max: Vec2)->Self{
		Self { min, max }
	}
	pub fn lerp(&self, param: f32)->Vec2{
		Vec2::lerp(self.min, self.max, param)
	}
	pub fn random(&self)->Vec2{
		let random = rand::random();
		self.lerp(random)
	}

	pub fn lerp_per_component(&self, per_param: Vec2)->Vec2{
		vec2(
			f32_lerp(self.min.x, self.max.x, per_param.x),
			f32_lerp(self.min.y, self.max.y, per_param.y),
		)
	}
	pub fn random_per_component(&self)->Vec2{
		let random_vec = vec2(rand::random(), rand::random());
		self.lerp_per_component(random_vec)
	}
}

#[derive(Default, Clone,Copy)]
pub struct MinMaxVec3{
	pub min: Vec3,
	pub max: Vec3
}

impl MinMaxVec3{
	pub fn new(min: Vec3, max: Vec3)->Self{
		Self { min, max }
	}
	
	pub fn lerp(&self, param: f32)->Vec3{
		Vec3::lerp(self.min, self.max, param)
	}
	pub fn random(&self)->Vec3{
		let random = rand::random();
		self.lerp(random)
	}

	pub fn lerp_per_component(&self, per_param: Vec3)->Vec3{
		vec3(
			f32_lerp(self.min.x, self.max.x, per_param.x),
			f32_lerp(self.min.y, self.max.y, per_param.y),
			f32_lerp(self.min.z, self.max.z, per_param.z)
		)
	}
	pub fn random_per_component(&self)->Vec3{
		let random_vec = vec3(rand::random(), rand::random(), rand::random());
		self.lerp_per_component(random_vec)
	}
}