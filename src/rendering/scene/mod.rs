use super::{camera::PerspectiveCamera, transform::Transform};
use glam::{Mat3, Mat4};

pub fn normal_mat(mat: Mat4) -> Mat3 {
    Mat3::from_mat4(mat).inverse().transpose()
}

pub trait SceneObject {
    fn transform(&self) -> &Transform;
    fn parent(&self) -> Option<&Self>;

    fn model_mat(&self) -> Mat4 {
        let mut mat = self.transform().compute_matrix();
        let mut parent = self.parent();
        while parent.is_some() {
            mat = parent.unwrap().transform().compute_matrix() * mat;
            parent = parent.unwrap().parent();
        }
        mat
    }

    fn model_view_mat(&self, camera: &PerspectiveCamera) -> Mat4 {
        camera.view_mat() * self.model_mat()
    }

    fn model_view_proj_mat(&self, camera: &PerspectiveCamera) -> Mat4 {
        camera.view_proj_mat() * self.model_mat()
    }

    fn model_normal_mat(&self) -> Mat3 {
        normal_mat(self.model_mat())
    }

    fn view_normal_mat(&self, camera: &PerspectiveCamera) -> Mat3 {
        Mat3::from_mat4(self.model_view_mat(camera))
            .inverse()
            .transpose()
    }
}

impl SceneObject for Transform {
    fn transform(&self) -> &Transform {
        self
    }
    fn parent(&self) -> Option<&Self> {
        None
    }
}
