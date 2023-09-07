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

pub fn has_intersection_ray_sphere(r: &Ray, s: &Sphere) -> bool {
    let oc = r.origin - s.center;
    let a = r.direction.length_squared();
    let half_b = oc.dot(r.direction);
    let c = oc.length_squared() - s.radius * s.radius;
    let discriminant = half_b * half_b - a * c;

    discriminant > 0.0
}

pub fn intersect_ray_sphere_within(r: &Ray, s: &Sphere, min: f32, max: f32) -> f32 {
    let oc = r.origin - s.center;
    let a = r.direction.length_squared();
    let half_b = oc.dot(r.direction);
    let c = oc.length_squared() - s.radius * s.radius;
    let discriminant = half_b * half_b - a * c;

    if discriminant < 0.0 {
        -1.0
    } else {
        let dsqrt = discriminant.sqrt();
        let mut t = (-half_b - dsqrt) / a;
        if t >= min && t <= max {
            t
        } else {
            t = (-half_b + dsqrt) / a;
            if t >= min && t <= max {
                t
            } else {
                -1.0
            }
        }
    }
}

pub fn intersect_ray_sphere(r: &Ray, s: &Sphere) -> f32 {
    intersect_ray_sphere_within(r, s, 0.0, std::f32::INFINITY)
}

pub fn has_intersection_normalized_ray_sphere(r: &Ray, s: &Sphere) -> bool {
    let oc = r.origin - s.center;
    let half_b = oc.dot(r.direction);
    let c = oc.length_squared() - s.radius * s.radius;
    let discriminant = half_b * half_b - c;

    discriminant > 0.0
}

pub fn intersect_normalized_ray_sphere_within(r: &Ray, s: &Sphere, min: f32, max: f32) -> f32 {
    let oc = r.origin - s.center;
    let half_b = oc.dot(r.direction);
    let c = oc.length_squared() - s.radius * s.radius;
    let discriminant = half_b * half_b - c;

    if discriminant < 0.0 {
        -1.0
    } else {
        let dsqrt = discriminant.sqrt();
        let mut t = -half_b - dsqrt;
        if t >= min && t <= max {
            t
        } else {
            t = -half_b + dsqrt;
            if t >= min && t <= max {
                t
            } else {
                -1.0
            }
        }
    }
}

pub fn intersect_normalized_ray_sphere(r: &Ray, s: &Sphere) -> f32 {
    intersect_normalized_ray_sphere_within(r, s, 0.0, std::f32::INFINITY)
}
