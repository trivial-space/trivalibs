//! # Binding System
//!
//! This module defines the binding system for shaders in the painter.
//!
//! ## Binding Types
//!
//! There are two main categories of bindings:
//!
//! - **ValueBindings**: Uniforms (buffers) and samplers that can be bound to shaders
//! - **LayerBindings**: Texture layers that can be bound as inputs to shaders
//!
//! ## Binding Override Hierarchy
//!
//! Bindings can be set at three levels, with later levels overriding earlier ones:
//!
//! 1. **Layer level** (lowest priority): Set via `Layer::set_layer_bindings()`
//! 2. **Shape/Effect level** (medium priority): Set when creating shapes/effects via `bindings` parameter
//! 3. **Instance level** (highest priority): Set via `InstanceBinding` for per-instance variation
//!
//! ### Override Rules:
//!
//! - Shape/Effect bindings override Layer bindings at the same binding slot
//! - Instance bindings override both Shape/Effect and Layer bindings at the same binding slot
//!
//! ## Instance Rendering Optimization
//!
//! The system optimizes rendering based on which binding types vary per-instance:
//!
//! - **No instances**: Set all bindings once, single draw call
//! - **Only value bindings vary**: Set layer bindings once, iterate through value bindings
//! - **Only layer bindings vary**: Set value bindings once, iterate through layer bindings
//! - **Both vary**: Iterate through all instances, setting both binding types per draw call
//!
//! This optimization is handled automatically by:
//! - `ValuesBindGroupData::from_bindings()` and `LayerBindGroupData::from_bindings()` in bind_group.rs
//! - `render_shape()` and `render_effect()` in painter.rs

use crate::{Painter, layer::Layer, painter::get_padded_size, sampler::Sampler};
use trivalibs_core::glam::{Mat3, Mat3A, Vec3, Vec3A};
use wgpu::{BindingType, ShaderStages};

#[derive(Clone, Copy)]
pub struct BindingLayout {
	pub(crate) binding_type: BindingType,
	pub(crate) visibility: ShaderStages,
}

#[derive(Clone, Copy)]
pub struct LayerLayout {
	pub(crate) visibility: ShaderStages,
}

#[derive(Clone, Copy)]
pub enum ValueBinding {
	Buffer(Buffer),
	Sampler(Sampler),
}

#[derive(Clone, Copy)]
pub enum LayerBinding {
	Source(Layer),
	AtIndex(Layer, usize),
	SourceAtMipLevel(Layer, u32),
	Depth(Layer),
}

#[derive(Clone, Copy)]
pub struct Buffer(pub(crate) usize);

impl Buffer {
	pub fn binding(&self) -> ValueBinding {
		ValueBinding::Buffer(*self)
	}
}

#[derive(Clone, Copy)]
pub struct BindingBuffer<T> {
	buffer: Buffer,
	t: std::marker::PhantomData<T>,
}

impl<T> BindingBuffer<T>
where
	T: bytemuck::Pod,
{
	pub fn new(painter: &mut Painter, data: T) -> Self {
		let buffer = painter.device.create_buffer(&wgpu::BufferDescriptor {
			label: None,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			size: get_padded_size(std::mem::size_of::<T>() as u64),
			mapped_at_creation: false,
		});

		painter.buffers.push(buffer);

		let buffer = Buffer(painter.buffers.len() - 1);

		let binding = BindingBuffer {
			buffer,
			t: std::marker::PhantomData,
		};

		binding.update(&painter, data);

		binding
	}

	pub fn update(&self, painter: &Painter, data: T) {
		let buffer = &painter.buffers[self.buffer.0];
		painter
			.queue
			.write_buffer(buffer, 0, bytemuck::cast_slice(&[data]));
	}

	pub fn binding(&self) -> ValueBinding {
		ValueBinding::Buffer(self.buffer)
	}
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable)]
pub struct Mat3U(pub(crate) Mat3A);
unsafe impl bytemuck::Pod for Mat3U {}

impl BindingBuffer<Mat3U> {
	pub fn new_mat3(painter: &mut Painter, data: Mat3) -> Self {
		BindingBuffer::new(painter, Mat3U(Mat3A::from(data)))
	}

	pub fn update_mat3(&self, painter: &Painter, data: Mat3) {
		self.update(painter, Mat3U(Mat3A::from(data)));
	}
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable)]
pub struct Vec3U(pub(crate) Vec3A);
unsafe impl bytemuck::Pod for Vec3U {}

impl BindingBuffer<Vec3U> {
	pub fn new_vec3(painter: &mut Painter, data: Vec3) -> Self {
		BindingBuffer::new(painter, Vec3U(Vec3A::from(data)))
	}

	pub fn update_vec3(&self, painter: &Painter, data: Vec3) {
		self.update(painter, Vec3U(Vec3A::from(data)));
	}
}

/// Per-instance binding overrides.
///
/// Allows individual instances of shapes or effects to override the base bindings
/// set at the layer or shape/effect level.
///
/// # Fields
///
/// - `bindings`: Value bindings (buffers, samplers) to override, indexed by binding slot
/// - `layers`: Layer bindings (textures) to override, indexed by binding slot
///
/// # Override Behavior
///
/// - If `bindings` is empty for all instances, value bindings won't vary per-instance
/// - If `layers` is empty for all instances, layer bindings won't vary per-instance
/// - If both are populated in at least one instance, both binding types will be set per draw call
///
/// # Examples
///
/// ```ignore
/// // Instance with only value binding override (uniform color per instance)
/// let instance1 = InstanceBinding {
///     bindings: vec![(0, color_buffer.binding())],
///     layers: vec![],
/// };
///
/// // Instance with only layer binding override (different texture per instance)
/// let instance2 = InstanceBinding {
///     bindings: vec![],
///     layers: vec![(0, LayerBinding::Source(layer))],
/// };
///
/// // Instance with both overrides
/// let instance3 = InstanceBinding {
///     bindings: vec![(0, color_buffer.binding())],
///     layers: vec![(0, LayerBinding::Source(layer))],
/// };
/// ```
#[derive(Clone)]
pub struct InstanceBinding {
	pub bindings: Vec<(u32, ValueBinding)>,
	pub layers: Vec<(u32, LayerBinding)>,
}

impl Default for InstanceBinding {
	fn default() -> Self {
		Self {
			bindings: Vec::with_capacity(0),
			layers: Vec::with_capacity(0),
		}
	}
}
