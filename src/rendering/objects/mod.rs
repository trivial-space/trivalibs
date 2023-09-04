use glam::Vec3;

use crate::prelude::Transform;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + t * self.direction
    }
}

pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
}

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

pub enum Axis {
    X,
    Y,
    Z,
}

pub struct Extend {
    pub axis: Axis,
    pub amount: f32,
}

pub struct Bound {
    pub center: Vec3,
    pub radius: f32,
    pub dimensions: Vec<Extend>,
    pub dominant_axis: Option<Axis>,
}

impl Bound {
    pub fn is_behind(&self, _transform: &Transform, _plane: &Plane) -> bool {
        todo!("Implement culling")
    }

    pub fn intersects_ray(&self, _transform: &Transform, _ray: &Ray) -> bool {
        todo!("Implement ray intersection")
    }
}

pub fn intersection_ray_sphere(r: &Ray, s: &Sphere) -> f32 {
    let oc = r.origin - s.center;
    let a = r.direction.length_squared();
    let half_b = oc.dot(r.direction);
    let c = oc.length_squared() - s.radius * s.radius;
    let discriminant = half_b * half_b - a * c;

    if discriminant < 0.0 {
        -1.0
    } else {
        (-half_b - discriminant.sqrt()) / a
    }
}
