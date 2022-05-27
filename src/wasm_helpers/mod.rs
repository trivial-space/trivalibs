use glam::{Mat2, Mat3, Mat4, Vec2, Vec3, Vec4};
use js_sys::Float32Array;

pub fn mat2_to_js(mat: &Mat2) -> Float32Array {
    js_sys::Float32Array::from(mat.to_cols_array().as_slice())
}

pub fn mat3_to_js(mat: &Mat3) -> Float32Array {
    js_sys::Float32Array::from(mat.to_cols_array().as_slice())
}

pub fn mat4_to_js(mat: &Mat4) -> Float32Array {
    js_sys::Float32Array::from(mat.to_cols_array().as_slice())
}

pub fn vec2_to_js(vec: &Vec2) -> Float32Array {
    js_sys::Float32Array::from(vec.to_array().as_slice())
}

pub fn vec3_to_js(vec: &Vec3) -> Float32Array {
    js_sys::Float32Array::from(vec.to_array().as_slice())
}

pub fn vec4_to_js(vec: &Vec4) -> Float32Array {
    js_sys::Float32Array::from(vec.to_array().as_slice())
}
