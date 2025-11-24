use crate::{
	data::{
		Position3D,
		grid::{CoordOpsFn, Grid},
		vertex_index::VertIdx3f,
	},
	rendering::BufferedGeometry,
	rendering::webgl_buffered_geometry::{
		RenderingPrimitive, VertexFormat, VertexType, WebglBufferedGeometry, WebglVertexData,
		create_buffered_geometry_layout,
	},
	utils::default,
};
use glam::Vec3;
use std::collections::{BTreeMap, HashMap};

pub mod utils;

#[derive(Debug, Clone)]
pub struct FaceVertex<V> {
	pub position_index: usize,
	pub data: V,
}

#[derive(Debug)]
pub struct Face<V> {
	pub vertices: [V; 4],
	pub position_indices: [usize; 4],
	pub vertex_count: u8,
	pub face_normal: Option<Vec3>,
	pub section: usize,
}

impl<V: bytemuck::Zeroable> Face<V> {
	fn new(face_vertices: Vec<FaceVertex<V>>, face_normal: Option<Vec3>, section: usize) -> Self {
		debug_assert!(face_vertices.len() == 3 || face_vertices.len() == 4);
		let vertex_count = face_vertices.len() as u8;

		let mut vertices = [V::zeroed(), V::zeroed(), V::zeroed(), V::zeroed()];
		let mut position_indices = [0, 0, 0, 0];

		for (i, fv) in face_vertices.into_iter().enumerate() {
			vertices[i] = fv.data;
			position_indices[i] = fv.position_index;
		}

		Self {
			vertices,
			position_indices,
			vertex_count,
			face_normal,
			section,
		}
	}
}

impl<V> Face<V> {
	pub fn vertex_data(&self) -> &[V] {
		&self.vertices[..self.vertex_count as usize]
	}

	pub fn vertex_data_mut(&mut self) -> &mut [V] {
		&mut self.vertices[..self.vertex_count as usize]
	}

	pub fn position_indices(&self) -> &[usize] {
		&self.position_indices[..self.vertex_count as usize]
	}
}

impl<V: Clone> Face<V> {
	pub fn vertices(&self) -> Vec<V> {
		self.vertex_data().to_vec()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionFaceRef {
	pub face_index: usize,
	pub vertex_slot: usize,
}

#[derive(Debug)]
pub struct VertexPosition {
	pub position: Vec3,
	pub faces: Vec<PositionFaceRef>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FaceProps {
	pub normal: Option<Vec3>,
	pub section: Option<usize>,
}

pub fn face_normal(normal: Vec3) -> FaceProps {
	FaceProps {
		normal: Some(normal),
		..default()
	}
}

pub fn face_section(section: usize) -> FaceProps {
	FaceProps {
		section: Some(section),
		..default()
	}
}

pub fn face_props(normal: Vec3, section: usize) -> FaceProps {
	FaceProps {
		normal: Some(normal),
		section: Some(section),
	}
}

pub const DEFAULT_MESH_SECTION: usize = 0;

pub struct MeshGeometry<V>
where
	V: Position3D,
{
	pub positions: Vec<VertexPosition>,
	pub faces: Vec<Face<V>>,
	position_indices: HashMap<VertIdx3f, usize>,
}

impl<V> MeshGeometry<V>
where
	V: Position3D + Clone + bytemuck::Zeroable,
{
	pub fn new() -> Self {
		Self {
			positions: Vec::new(),
			faces: Vec::new(),
			position_indices: HashMap::new(),
		}
	}

	pub fn position_count(&self) -> usize {
		self.positions.len()
	}

	pub fn face_count(&self) -> usize {
		self.faces.len()
	}

	fn get_position_index(&mut self, pos: Vec3) -> usize {
		if let Some(idx) = self.position_indices.get(&pos.into()) {
			*idx
		} else {
			let idx = self.positions.len();
			self.positions.push(VertexPosition {
				position: pos,
				faces: Vec::with_capacity(8),
			});
			self.position_indices.insert(pos.into(), idx);
			idx
		}
	}

	fn register_face_refs(&mut self, face_idx: usize) {
		let face = &self.faces[face_idx];
		for (slot, &pos_idx) in face.position_indices().iter().enumerate() {
			self.positions[pos_idx].faces.push(PositionFaceRef {
				face_index: face_idx,
				vertex_slot: slot,
			});
		}
	}

	fn unregister_face_refs(&mut self, face_idx: usize, face: &Face<V>) {
		for &pos_idx in face.position_indices() {
			self.positions[pos_idx]
				.faces
				.retain(|f| f.face_index != face_idx);
		}
	}

	fn rewrite_face_refs(&mut self, from: usize, to: usize) {
		let face = &self.faces[to];
		for &pos_idx in face.position_indices() {
			if let Some(face_ref) = self.positions[pos_idx]
				.faces
				.iter_mut()
				.find(|r| r.face_index == from)
			{
				face_ref.face_index = to;
			}
		}
	}

	fn append_face(&mut self, vertices: Vec<FaceVertex<V>>, props: FaceProps, section_idx: usize) {
		let face_idx = self.faces.len();
		self.faces
			.push(Face::new(vertices, props.normal, section_idx));
		self.register_face_refs(face_idx);
	}

	fn position_face_data(&self, vertex_idx: usize) -> &V {
		let vertex = &self.positions[vertex_idx];
		let reference = vertex
			.faces
			.first()
			.expect("vertex must be part of at least one face to compact");
		let face = &self.faces[reference.face_index];
		&face.vertices[reference.vertex_slot]
	}

	pub fn add_face_data(&mut self, verts: &[V], mut props: FaceProps) {
		debug_assert!(verts.len() == 3 || verts.len() == 4);
		let section_idx = props.section.unwrap_or(DEFAULT_MESH_SECTION);
		props.section = None;
		let vertices = verts
			.iter()
			.map(|v| {
				let idx = self.get_position_index(v.position());
				FaceVertex {
					position_index: idx,
					data: v.clone(),
				}
			})
			.collect();
		self.append_face(vertices, props, section_idx);
	}

	pub fn add_face(&mut self, verts: &[V]) {
		self.add_face_data(verts, default())
	}

	pub fn add_grid_ccw_quads_data<A: CoordOpsFn>(&mut self, grid: &Grid<V, A>, props: FaceProps) {
		for quad in grid.to_ccw_quads() {
			self.add_face_data(&quad, props);
		}
	}

	pub fn add_grid_ccw_quads<A: CoordOpsFn>(&mut self, grid: &Grid<V, A>) {
		self.add_grid_ccw_quads_data(grid, default())
	}

	pub fn add_grid_cw_quads_data<A: CoordOpsFn>(&mut self, grid: &Grid<V, A>, props: FaceProps) {
		for quad in grid.to_cw_quads() {
			self.add_face_data(&quad, props);
		}
	}

	pub fn add_grid_cw_quads<A: CoordOpsFn>(&mut self, grid: &Grid<V, A>) {
		self.add_grid_cw_quads_data(grid, default())
	}

	pub fn new_from_section(&self, section_idx: usize) -> Self {
		let mut geom = MeshGeometry::new();
		for face in self.faces.iter().filter(|face| face.section == section_idx) {
			let props = FaceProps {
				normal: face.face_normal,
				section: Some(DEFAULT_MESH_SECTION),
			};
			let vertices = face.vertices();
			geom.add_face_data(&vertices, props);
		}
		geom
	}

	pub fn split_by_sections(&self) -> BTreeMap<usize, MeshGeometry<V>> {
		let mut result = BTreeMap::new();
		for face in &self.faces {
			let entry = result.entry(face.section).or_insert_with(MeshGeometry::new);
			let props = FaceProps {
				normal: face.face_normal,
				section: Some(DEFAULT_MESH_SECTION),
			};
			let vertices = face.vertices();
			entry.add_face_data(&vertices, props);
		}
		result
	}

	pub fn vertex(&self, index: usize) -> &VertexPosition {
		&self.positions[index]
	}

	pub fn get_vertex_index(&self, pos: Vec3) -> Option<usize> {
		self.position_indices.get(&pos.into()).copied()
	}

	pub fn face(&self, index: usize) -> &Face<V> {
		&self.faces[index]
	}

	pub fn face_mut(&mut self, index: usize) -> &mut Face<V> {
		&mut self.faces[index]
	}

	pub fn remove_face(&mut self, face_idx: usize) {
		if face_idx >= self.faces.len() {
			return;
		}

		let last_idx = self.faces.len() - 1;
		let removed_face = self.faces.swap_remove(face_idx);
		self.unregister_face_refs(face_idx, &removed_face);

		if face_idx != last_idx {
			self.rewrite_face_refs(last_idx, face_idx);
		}
	}

	fn calculate_vertex_normal(&self, vertex_idx: usize, section_filter: Option<usize>) -> Vec3 {
		let vert = &self.positions[vertex_idx];
		let mut normal = Vec3::ZERO;
		for face_ref in vert.faces.iter() {
			let face = &self.faces[face_ref.face_index];
			if section_filter.map_or(true, |section| face.section == section) {
				if let Some(face_normal) = face.face_normal {
					normal += face_normal;
				}
			}
		}
		normal.normalize_or_zero()
	}

	fn calculate_face_normal(&self, face_idx: usize) -> Vec3 {
		let face = &self.faces[face_idx];
		let verts = face.vertex_data();
		let pos0 = verts[0].position();
		let pos1 = verts[1].position();
		let pos2 = verts[2].position();

		let mut v1 = pos2 - pos0;
		let mut v2 = pos1 - pos0;

		let v1_len = v1.length();
		let v2_len = v2.length();
		let v1_len_0 = v1_len < 0.0001;
		let v2_len_0 = v2_len < 0.0001;
		let v3_len_0 = (v2 / v2_len).dot(v1 / v1_len).abs() > 0.9999;

		if (v1_len_0 || v2_len_0 || v3_len_0) && verts.len() > 3 {
			if v2_len_0 {
				v2 = pos1 - verts[3].position();
			} else {
				v1 = verts[3].position() - pos0;
			}
		}

		v2.cross(v1).normalize()
	}

	fn ensure_face_normals(&mut self) -> bool {
		let mut has_quads = false;
		for face_idx in 0..self.faces.len() {
			let needs_normal = {
				let face = &self.faces[face_idx];
				if !has_quads && face.vertex_count > 3 {
					has_quads = true;
				}
				face.face_normal.is_none()
			};

			if needs_normal {
				let normal = Some(self.calculate_face_normal(face_idx));
				self.faces[face_idx].face_normal = normal;
			}
		}
		has_quads
	}

	fn has_quads(&self) -> bool {
		self.faces.iter().any(|face| face.vertex_count > 3)
	}
}

#[derive(PartialEq)]
pub enum MeshBufferType {
	/// Emit per-face vertex data without normals; triangles/quads unrolled in order added.
	/// Assumes provided vertex data might differ even for shared positions.
	FaceVertices,
	/// Emit per-face vertex data with interpolated vertex normals for smooth shading.
	/// Assumes provided vertex data might differ even for shared positions.
	FaceVerticesWithVertexNormals,
	/// Emit per-face vertex data with the same normal duplicated per face for flat shading.
	/// Vertex data might differ for shared positions because normals might be different.
	FaceVerticesWithFaceNormals,
	/// Emit deduped vertex data (no normals) plus an index buffer. Most compact buffer storage.
	/// Assumes provided vertex data is the same for shared positions.
	CompactVertices,
	/// Same as `CompactVertices` but appends a vertex normal alongside each deduped vertex.
	/// Assumes provided vertex data is the same for shared positions.
	CompactVerticesWithNormal,
}

impl MeshBufferType {
	fn includes_normals(&self) -> bool {
		matches!(
			self,
			MeshBufferType::FaceVerticesWithVertexNormals
				| MeshBufferType::FaceVerticesWithFaceNormals
				| MeshBufferType::CompactVerticesWithNormal
		)
	}
}

impl<V> MeshGeometry<V>
where
	V: Position3D + Clone + bytemuck::Pod,
{
	pub fn to_buffered_geometry_by_type(&mut self, geom_type: MeshBufferType) -> BufferedGeometry {
		let mut buffer = vec![];
		let mut indices = vec![];
		let mut vertex_count = 0;

		match geom_type {
			MeshBufferType::FaceVertices => {
				let has_quads = self.has_quads();
				for face in self.faces.iter() {
					let index_offset = vertex_count;
					for vertex_data in face.vertex_data() {
						buffer.extend(bytemuck::bytes_of(vertex_data));
						vertex_count += 1;
					}
					if has_quads {
						let face_indices = (0..face.vertex_count as usize)
							.map(|i| i + index_offset)
							.collect::<Vec<_>>();
						fill_face_index_buffer(&mut indices, &face_indices);
					}
				}
			}

			MeshBufferType::FaceVerticesWithVertexNormals => {
				let has_quads = self.ensure_face_normals();
				for face in self.faces.iter() {
					let index_offset = vertex_count;
					for (vertex_data, &pos_idx) in
						face.vertex_data().iter().zip(face.position_indices())
					{
						let normal = self.calculate_vertex_normal(pos_idx, Some(face.section));
						buffer.extend(bytemuck::bytes_of(vertex_data));
						buffer.extend(bytemuck::bytes_of(&normal));
						vertex_count += 1;
					}
					if has_quads {
						let face_indices = (0..face.vertex_count as usize)
							.map(|i| i + index_offset)
							.collect::<Vec<_>>();
						fill_face_index_buffer(&mut indices, &face_indices);
					}
				}
			}

			MeshBufferType::FaceVerticesWithFaceNormals => {
				let has_quads = self.ensure_face_normals();
				for face in self.faces.iter() {
					let index_offset = vertex_count;
					let normal = face.face_normal.unwrap();
					for vertex_data in face.vertex_data() {
						buffer.extend(bytemuck::bytes_of(vertex_data));
						buffer.extend(bytemuck::bytes_of(&normal));
						vertex_count += 1;
					}
					if has_quads {
						let face_indices = (0..face.vertex_count as usize)
							.map(|i| i + index_offset)
							.collect::<Vec<_>>();
						fill_face_index_buffer(&mut indices, &face_indices);
					}
				}
			}

			MeshBufferType::CompactVertices => {
				for vertex_idx in 0..self.positions.len() {
					let data = self.position_face_data(vertex_idx);
					buffer.extend(bytemuck::bytes_of(data));
					vertex_count += 1;
				}

				for face in self.faces.iter() {
					fill_face_index_buffer(&mut indices, face.position_indices());
				}
			}

			MeshBufferType::CompactVerticesWithNormal => {
				self.ensure_face_normals();
				for vertex_idx in 0..self.positions.len() {
					let data = self.position_face_data(vertex_idx);
					let normal = self.calculate_vertex_normal(vertex_idx, None);
					buffer.extend(bytemuck::bytes_of(data));
					buffer.extend(bytemuck::bytes_of(&normal));
					vertex_count += 1;
				}

				for face in self.faces.iter() {
					fill_face_index_buffer(&mut indices, face.position_indices());
				}
			}
		}

		let indices_len = indices.len();

		BufferedGeometry {
			vertex_buffer: buffer,
			index_buffer: if indices_len == 0 {
				None
			} else {
				Some(indices)
			},
			vertex_count: vertex_count as u32,
			index_count: (indices_len / 4) as u32,
		}
	}
}

fn fill_face_index_buffer(index_buffer: &mut Vec<u8>, indices: &[usize]) {
	index_buffer.extend(bytemuck::cast_slice(&[
		indices[0] as u32,
		indices[1] as u32,
		indices[2] as u32,
	]));

	if indices.len() == 4 {
		index_buffer.extend(bytemuck::cast_slice(&[
			indices[0] as u32,
			indices[2] as u32,
			indices[3] as u32,
		]));
	}
}

impl<V> MeshGeometry<V>
where
	V: WebglVertexData + Position3D + Clone + bytemuck::Zeroable,
{
	pub fn to_webgl_buffered_geometry_by_type(
		&mut self,
		geom_type: MeshBufferType,
	) -> WebglBufferedGeometry {
		let mut layout: Vec<VertexType> = V::vertex_layout();

		if geom_type.includes_normals() {
			layout.push(VertexType {
				name: "normal",
				format: VertexFormat::Float32x3,
			})
		}

		let buffer = self.to_buffered_geometry_by_type(geom_type);

		let geom_layout = create_buffered_geometry_layout(layout);

		WebglBufferedGeometry {
			rendering_primitive: RenderingPrimitive::Triangles,
			vertex_count: if buffer.index_buffer.is_some() {
				buffer.index_count
			} else {
				buffer.vertex_count
			},
			vertex_size: geom_layout.vertex_size,
			vertex_layout: geom_layout.vertex_layout,
			buffer: buffer.vertex_buffer,
			indices: buffer.index_buffer,
		}
	}
}

impl Position3D for VertexPosition {
	fn position(&self) -> Vec3 {
		self.position
	}
}

#[cfg(test)]
mod tests;
