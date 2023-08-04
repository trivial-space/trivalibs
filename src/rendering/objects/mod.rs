use glam::Vec3;

use crate::prelude::Transform;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,
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
