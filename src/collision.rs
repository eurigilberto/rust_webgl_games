#![allow(dead_code)]

use glam::*;

use crate::math::cross_vec2;

#[derive(Clone, Copy)]
pub struct Sphere {
    pub position: Vec3,
    pub radius: f32,
}

#[derive(Clone, Copy)]
pub struct AxisAlignedBoundingBox {
    pub position: Vec3,
    pub size: Vec3,
}

#[derive(Clone, Copy)]
pub struct OrientedBoundingBox {
    pub axis: Mat3,
    pub aligned_box: AxisAlignedBoundingBox,
}

#[derive(Clone, Copy)]
pub enum SimpleCollider {
    Sphere(Sphere),
    AxisAlignedBoundingBox(AxisAlignedBoundingBox),
    OrientedBoundingBox(OrientedBoundingBox),
}

pub struct CollisionResult {
    pub correction_dir: Vec3,
    pub distance: f32,
}

pub fn compute_simple_collision(a: &SimpleCollider, b: &SimpleCollider) -> Option<CollisionResult> {
    match (a, b) {
        (SimpleCollider::Sphere(s0), SimpleCollider::Sphere(s1)) => sphere_collision(s0, s1),
        (SimpleCollider::Sphere(s), SimpleCollider::AxisAlignedBoundingBox(aabb)) => {
            sphere_aabb_collision(s, aabb)
        }
        (SimpleCollider::Sphere(s), SimpleCollider::OrientedBoundingBox(obb)) => {
            sphere_obb_collision(s, obb)
        }
        _ => None,
    }
}

pub struct Plane {
    pub center: Vec3,
    pub normal: Vec3,
}

pub fn ray_to_plane_intersection(ray: &Ray, plane: &Plane) -> Option<f32> {
    let dot_val = ray.direction.dot(plane.normal);
    if dot_val.abs() > 0.001 {
        let result = (plane.center - ray.position).dot(plane.normal) / dot_val;
        if result >= 0.0 {
            return Some(result)
        }
    }
    None
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Ray {
    pub position: Vec3,
    pub direction: Vec3,
    pub distance: Option<f32>,
}

impl Ray {
    pub fn get_position(&self, param: f32) -> Vec3 {
        self.position + self.direction * param
    }
}

pub fn ray_to_ray_closest_point(ray_a: &Ray, ray_b: &Ray) -> Option<(f32, f32)> {
    if ray_a.direction.dot(ray_b.direction).abs() > 0.99 {
        None
    } else {
        let ab_cross = ray_a.direction.cross(ray_b.direction);

        let norm_b = ab_cross.cross(ray_b.direction);
        let plane_b = Plane {
            center: ray_b.position,
            normal: norm_b,
        };

        let ta = ray_to_plane_intersection(ray_a, &plane_b);

        let norm_a = ab_cross.cross(ray_a.direction);
        let plane_a = Plane {
            center: ray_a.position,
            normal: norm_a,
        };

        let tb = ray_to_plane_intersection(ray_b, &plane_a);

        match (ta, tb) {
            (Some(ta), Some(tb)) => Some((ta, tb)),
            _ => None,
        }
    }
}

pub enum RayConvexResult {
    Single(Vec3),
    Double { near: Vec3, far: Vec3 },
}

pub fn ray_sphere_intersection(ray: &Ray, sphere: &Sphere) -> Option<RayConvexResult> {
    let ray_to_sphere = sphere.position - ray.position;
    let distance_to_pos = ray_to_sphere.dot(ray.direction);
    let closest_point_center = ray.direction * distance_to_pos + ray.position;
    let distance_to_center = (sphere.position - closest_point_center).length();
    if distance_to_center <= sphere.radius {
        if (sphere.radius - distance_to_center) < 0.001 {
            Some(RayConvexResult::Single(closest_point_center))
        } else {
            let intersection_distance =
                f32::sqrt(sphere.radius.powi(2) - distance_to_center.powi(2));
            let near = ray.direction * (distance_to_pos - intersection_distance) + ray.position;
            let far = ray.direction * (distance_to_pos + intersection_distance) + ray.position;
            Some(RayConvexResult::Double { near, far })
        }
    } else {
        None
    }
}

pub fn sphere_offset_aabb_collision(
    offset_pos: Vec3,
    radius: f32,
    size: Vec3,
) -> Option<CollisionResult> {
    fn collision_check(offset: f32, half_size: f32, radius: f32) -> Option<f32> {
        let abs_offset_x = f32::abs(offset);
        if abs_offset_x <= half_size {
            Some((half_size - abs_offset_x + radius) * f32::signum(offset))
        } else if abs_offset_x - half_size <= radius {
            Some((radius - (abs_offset_x - half_size)) * f32::signum(offset))
        } else {
            None
        }
    }

    let x_collision = collision_check(offset_pos.x, size.x * 0.5, radius);
    let y_collision = collision_check(offset_pos.y, size.y * 0.5, radius);
    let z_collision = collision_check(offset_pos.z, size.z * 0.5, radius);

    match (x_collision, y_collision, z_collision) {
        (Some(x_c), Some(y_c), Some(z_c)) => {
            let mut distances = vec![(x_c, Vec3::X), (y_c, Vec3::Y), (z_c, Vec3::Z)];

            let comparison = |a: &(f32, _), b: &(f32, _)| a.0.abs().total_cmp(&b.0.abs());

            distances.sort_by(comparison);
            Some(CollisionResult {
                correction_dir: distances[0].1 * f32::signum(distances[0].0),
                distance: f32::abs(distances[0].0),
            })
        }
        _ => None,
    }
}

pub fn sphere_aabb_collision(
    sphere: &Sphere,
    aabb: &AxisAlignedBoundingBox,
) -> Option<CollisionResult> {
    let offset_pos = sphere.position - aabb.position;

    sphere_offset_aabb_collision(offset_pos, sphere.radius, aabb.size)
}

pub fn sphere_obb_collision(sphere: &Sphere, obb: &OrientedBoundingBox) -> Option<CollisionResult> {
    let offset_pos = sphere.position - obb.aligned_box.position;
    let aligned_offset_pos = vec3(
        obb.axis.x_axis.dot(offset_pos),
        obb.axis.y_axis.dot(offset_pos),
        obb.axis.z_axis.dot(offset_pos),
    );

    match sphere_offset_aabb_collision(aligned_offset_pos, sphere.radius, obb.aligned_box.size) {
        Some(result) => {
            let c_dir = result.correction_dir;
            let dir =
                obb.axis.x_axis * c_dir.x + obb.axis.y_axis * c_dir.y + obb.axis.z_axis * c_dir.z;
            Some(CollisionResult {
                correction_dir: dir,
                distance: result.distance,
            })
        }
        None => None,
    }
}

pub fn sphere_collision(sphere_0: &Sphere, sphere_1: &Sphere) -> Option<CollisionResult> {
    let ab_vec = sphere_0.position - sphere_1.position;
    let distance = ab_vec.length();
    if distance < sphere_0.radius + sphere_1.radius {
        let ab_dir = if distance > f32::EPSILON * 2.0 {
            ab_vec / distance
        } else {
            Vec3::X
        };
        let distance = (sphere_0.radius + sphere_1.radius) - distance;
        Some(CollisionResult {
            correction_dir: ab_dir,
            distance,
        })
    } else {
        None
    }
}

pub fn point2d_in_triangle(point: Vec2, triangle: [Vec2; 3])->bool{
    let mut prev_val = None;
    for i in 0..3{
        let next_i = (i + 1) % 3;
        let to_point_vec = point - triangle[i];
        let edge_vec = triangle[next_i] - triangle[i];
        let value = cross_vec2(to_point_vec, edge_vec);
        let sign_val =  value >= 0.0;
        if prev_val.is_none(){
            prev_val = Some(sign_val);
        }else {
            if prev_val.unwrap() != sign_val{
                return false
            }
        }
    }
    return true
    
}