use super::{camera::PerspectiveCamera, transform::Transform};
use glam::{Mat3, Mat4};

pub trait SceneObject {
    fn transform(&self) -> &Transform;
    fn parent(&self) -> Option<&Transform>;
}

impl SceneObject for Transform {
    fn transform(&self) -> &Transform {
        self
    }
    fn parent(&self) -> Option<&Transform> {
        None
    }
}

pub fn view_mat(camera: &PerspectiveCamera) -> Mat4 {
    camera.transform().compute_matrix()
}

pub fn model_mat<O: SceneObject>(obj: &O) -> Mat4 {
    obj.transform().compute_matrix()
}

pub fn model_view_mat<O: SceneObject>(obj: &O, camera: &PerspectiveCamera) -> Mat4 {
    view_mat(camera) * model_mat(obj)
}
pub fn model_view_proj_mat<O: SceneObject>(obj: &O, camera: &PerspectiveCamera) -> Mat4 {
    camera.proj * model_view_mat(obj, camera)
}

pub fn model_normal_mat<O: SceneObject>(obj: &O) -> Mat3 {
    Mat3::from_mat4(model_mat(obj)).inverse().transpose()
}

pub fn view_normal_mat<O: SceneObject>(obj: &O, camera: &PerspectiveCamera) -> Mat3 {
    Mat3::from_mat4(model_view_mat(obj, camera))
        .inverse()
        .transpose()
}
