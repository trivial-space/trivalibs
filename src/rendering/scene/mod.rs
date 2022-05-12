use glam::Mat4;
use serde::Serialize;

use super::{buffered_geometry::BufferedGeometry, camera::PerspectiveCamera, transform::Transform};
use std::collections::HashMap;

#[derive(Default, Serialize)]
pub struct SceneObject {
    geometry: &'static str,
    transform: Transform,
}

#[derive(Default, Serialize)]
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
    pub fn model_view_proj_mat(&self, obj: &'static str) -> Mat4 {
        *self.proj_mat() * self.model_view_mat(obj)
    }

    pub fn model_normal_mat(&self, obj: &'static str) -> Mat4 {
        self.model_mat(obj).inverse().transpose()
    }
    pub fn view_normal_mat(&self, obj: &'static str) -> Mat4 {
        self.model_view_mat(obj).inverse().transpose()
    }

    pub fn set_obj(&mut self, key: &'static str, geometry: &'static str, transform: Transform) {
        self.objects.insert(
            key,
            SceneObject {
                geometry,
                transform,
            },
        );
    }

    pub fn set_geometry(&mut self, key: &'static str, geometry: BufferedGeometry) {
        self.geometries.insert(key, geometry);
    }

    pub fn update_cam<F: Fn(&mut PerspectiveCamera)>(&mut self, f: F) {
        f(&mut self.camera);
    }

    pub fn update_obj<F: Fn(&mut SceneObject)>(&mut self, obj: &'static str, f: F) {
        f(self.objects.get_mut(obj).unwrap())
    }
}
