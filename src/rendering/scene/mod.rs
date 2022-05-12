use glam::Mat4;

use super::{buffered_geometry::BufferedGeometry, camera::PerspectiveCamera, transform::Transform};
use std::collections::HashMap;

pub struct SceneObject {
    geometry: &'static str,
    transform: Transform,
}

pub struct Scene {
    geometries: HashMap<&'static str, BufferedGeometry>,
    objects: HashMap<&'static str, SceneObject>,
    camera: PerspectiveCamera,
}

impl Scene {
    pub fn proj_mat(&self) -> &Mat4 {
        &self.camera.proj
    }

    pub fn view_mat(&self) -> Mat4 {
        self.camera.transform.compute_matrix()
    }

    pub fn model_mat(&self, obj: &'static str) -> Mat4 {
        self.objects[obj].transform.compute_matrix()
    }

    pub fn model_view_mat(&self, obj: &'static str) -> Mat4 {
        self.view_mat() * self.model_mat(obj)
    }
    pub fn model_view_proj_mat(&self, obj: &'static str) {}
    pub fn model_normal_mat(&self, obj: &'static str) {}
    pub fn view_normal_mat(&self, obj: &'static str) {}

    pub fn set_obj(&mut self, key: &'static str, geometry: &'static str, transform: Transform) {}
    pub fn set_geometry(&mut self, key: &'static str, geometry: BufferedGeometry) {}

    pub fn update_cam<F: Fn(&mut PerspectiveCamera)>(&mut self, f: F) {}
    pub fn update_obj<F: Fn(&mut SceneObject)>(&mut self, obj: &'static str, f: F) {}
}
