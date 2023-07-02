use super::{camera::PerspectiveCamera, transform::Transform};
use glam::{Mat3, Mat4};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Default, Serialize)]
pub struct SceneObject {
    transform: Transform,
}

#[derive(Default, Serialize)]
pub struct Scene {
    objects: HashMap<&'static str, SceneObject>,
    camera: PerspectiveCamera,
}

impl Scene {
    pub fn new() -> Self {
        Scene::default()
    }

    pub fn obj(&self, key: &'static str) -> &SceneObject {
        self.objects.get(key).unwrap()
    }

    pub fn proj_mat(&self) -> &Mat4 {
        &self.camera.proj
    }

    pub fn view_mat(&self) -> Mat4 {
        self.camera.transform().compute_matrix()
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

    pub fn model_normal_mat(&self, obj: &'static str) -> Mat3 {
        Mat3::from_mat4(self.model_mat(obj)).inverse().transpose()
    }

    pub fn view_normal_mat(&self, obj: &'static str) -> Mat3 {
        Mat3::from_mat4(self.model_view_mat(obj))
            .inverse()
            .transpose()
    }

    pub fn set_obj(&mut self, key: &'static str, transform: Transform) {
        self.objects.insert(key, SceneObject { transform });
    }

    pub fn update_cam<F: Fn(&mut PerspectiveCamera)>(&mut self, f: F) {
        f(&mut self.camera);
    }

    pub fn update_obj_transform<F: Fn(&mut Transform)>(&mut self, obj: &'static str, f: F) {
        f(&mut self.objects.get_mut(obj).unwrap().transform)
    }
}
