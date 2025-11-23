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
	pub vertices: Vec<FaceVertex<V>>,
	pub face_normal: Option<Vec3>,
	pub section: usize,
}

impl<V> Face<V> {
	fn new(vertices: Vec<FaceVertex<V>>, face_normal: Option<Vec3>, section: usize) -> Self {
		debug_assert!(vertices.len() == 3 || vertices.len() == 4);
		Self {
			vertices,
			face_normal,
			section,
		}
	}
}

impl<V: Clone> Face<V> {
	pub fn vertices(&self) -> Vec<V> {
		self.vertices.iter().map(|v| v.data.clone()).collect()
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
	pub vertex_normal: Option<Vec3>,
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

pub struct MeshSection<V>
where
	V: Position3D,
{
	pub positions: Vec<VertexPosition>,
	faces: Vec<Face<V>>,
	position_indices: HashMap<VertIdx3f, usize>,
	section_index: usize,
}

impl<V> MeshSection<V>
where
	V: Position3D + Clone,
{
	pub fn new(section_index: usize) -> Self {
		Self {
			positions: Vec::new(),
			faces: Vec::new(),
			position_indices: HashMap::new(),
			section_index,
		}
	}

	fn get_position_index(&mut self, pos: Vec3) -> usize {
		if let Some(idx) = self.position_indices.get(&pos.into()) {
			*idx
		} else {
			let idx = self.positions.len();
			self.positions.push(VertexPosition {
				position: pos,
				faces: Vec::with_capacity(8),
				vertex_normal: None,
			});
			self.position_indices.insert(pos.into(), idx);
			idx
		}
	}

	fn register_face_refs(&mut self, face_idx: usize) {
		let refs = self.faces[face_idx]
			.vertices
			.iter()
			.enumerate()
			.map(|(slot, fv)| (slot, fv.position_index))
			.collect::<Vec<_>>();
		for (slot, vertex_idx) in refs {
			let vertex = &mut self.positions[vertex_idx];
			vertex.faces.push(PositionFaceRef {
				face_index: face_idx,
				vertex_slot: slot,
			});
		}
	}

	fn append_face(&mut self, vertices: Vec<FaceVertex<V>>, props: FaceProps) {
		let face_idx = self.faces.len();
		self.faces
			.push(Face::new(vertices, props.normal, self.section_index));
		self.register_face_refs(face_idx);
	}

	fn add_face_data(&mut self, verts: &[V], props: FaceProps) {
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
		self.append_face(vertices, props);
	}

	pub fn remove_face(&mut self, face_idx: usize) {
		if face_idx >= self.faces.len() {
			return;
		}

		let last_idx = self.faces.len() - 1;
		let face_vertex_indices = self.faces[face_idx]
			.vertices
			.iter()
			.map(|fv| fv.position_index)
			.collect::<Vec<_>>();

		for vertex_idx in face_vertex_indices {
			let vertex = &mut self.positions[vertex_idx];
			vertex.faces.retain(|f| f.face_index != face_idx);
		}

		self.faces.swap_remove(face_idx);

		if face_idx != last_idx {
			let moved_vertices = self.faces[face_idx]
				.vertices
				.iter()
				.map(|fv| fv.position_index)
				.collect::<Vec<_>>();
			for vertex_idx in moved_vertices {
				if let Some(face_ref) = self.positions[vertex_idx]
					.faces
					.iter_mut()
					.find(|r| r.face_index == last_idx)
				{
					face_ref.face_index = face_idx;
				}
			}
		}
	}

	fn position_face_data(&self, vertex_idx: usize) -> &V {
		let vertex = &self.positions[vertex_idx];
		let reference = vertex
			.faces
			.first()
			.expect("vertex must be part of at least one face to compact");
		let face = &self.faces[reference.face_index];
		&face.vertices[reference.vertex_slot].data
	}
}

pub const DEFAULT_MESH_SECTION: usize = 0;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct SectionIndex {
	pub section: usize,
	pub index: usize,
}

impl From<usize> for SectionIndex {
	fn from(index: usize) -> Self {
		Self {
			section: DEFAULT_MESH_SECTION,
			index,
		}
	}
}

pub fn section_index(section: usize, index: usize) -> SectionIndex {
	SectionIndex { section, index }
}

pub struct MeshGeometry<V>
where
	V: Position3D,
{
	pub sections: BTreeMap<usize, MeshSection<V>>,
}

impl<V> MeshGeometry<V>
where
	V: Position3D + Clone,
{
	pub fn new() -> Self {
		Self {
			sections: BTreeMap::new(),
		}
	}

	pub fn vertex_count(&self) -> usize {
		self.sections
			.values()
			.map(|section| section.positions.len())
			.sum()
	}

	pub fn face_count(&self) -> usize {
		self.sections
			.values()
			.map(|section| section.faces.len())
			.sum()
	}

	pub fn default_section(&self) -> &MeshSection<V> {
		self.sections
			.get(&DEFAULT_MESH_SECTION)
			.expect("default section missing")
	}

	pub fn get_or_create_section(&mut self, section_idx: usize) -> &mut MeshSection<V> {
		self.sections
			.entry(section_idx)
			.or_insert_with(|| MeshSection::new(section_idx))
	}

	pub fn add_face_data(&mut self, verts: &[V], mut props: FaceProps) {
		debug_assert!(verts.len() == 3 || verts.len() == 4);
		let section_idx = props.section.unwrap_or(DEFAULT_MESH_SECTION);
		props.section = None;
		self.get_or_create_section(section_idx)
			.add_face_data(verts, props);
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

	pub fn vertex<T: Into<SectionIndex>>(&self, into_idx: T) -> &VertexPosition {
		let i: SectionIndex = into_idx.into();
		&self.sections.get(&i.section).unwrap().positions[i.index]
	}

	pub fn get_vertex_index(&self, pos: Vec3) -> Vec<SectionIndex> {
		let mut indices = vec![];
		for (section_idx, section) in self.sections.iter() {
			if let Some(idx) = section.position_indices.get(&pos.into()) {
				indices.push(section_index(*section_idx, *idx));
			}
		}
		indices
	}

	pub fn face<T: Into<SectionIndex>>(&self, into_idx: T) -> &Face<V> {
		let i: SectionIndex = into_idx.into();
		&self.sections.get(&i.section).unwrap().faces[i.index]
	}

	pub fn face_mut<T: Into<SectionIndex>>(&mut self, into_idx: T) -> &mut Face<V> {
		let i: SectionIndex = into_idx.into();
		&mut self.sections.get_mut(&i.section).unwrap().faces[i.index]
	}

	pub fn remove_face<T: Into<SectionIndex>>(&mut self, into_idx: T) {
		let face_idx: SectionIndex = into_idx.into();
		self.sections
			.get_mut(&face_idx.section)
			.unwrap()
			.remove_face(face_idx.index);
	}

	fn calculate_vertex_normal<I: Into<SectionIndex>>(&self, into_idx: I) -> Vec3 {
		let idx: SectionIndex = into_idx.into();
		let section = self.sections.get(&idx.section).unwrap();
		let vert = &section.positions[idx.index];
		let mut normal = Vec3::ZERO;
		for face_ref in vert.faces.iter() {
			let face = &section.faces[face_ref.face_index];
			if let Some(face_normal) = face.face_normal {
				normal += face_normal;
			}
		}
		normal.normalize_or_zero()
	}

	fn calculate_face_normal<I: Into<SectionIndex>>(&self, into_idx: I) -> Vec3 {
		let idx: SectionIndex = into_idx.into();
		let section = self.sections.get(&idx.section).unwrap();
		let face = &section.faces[idx.index];

		let verts = &face.vertices;
		let pos0 = verts[0].data.position();
		let pos1 = verts[1].data.position();
		let pos2 = verts[2].data.position();

		let mut v1 = pos2 - pos0;
		let mut v2 = pos1 - pos0;

		let v1_len = v1.length();
		let v2_len = v2.length();
		let v1_len_0 = v1_len < 0.0001;
		let v2_len_0 = v2_len < 0.0001;
		let v3_len_0 = (v2 / v2_len).dot(v1 / v1_len).abs() > 0.9999;

		if (v1_len_0 || v2_len_0 || v3_len_0) && verts.len() > 3 {
			if v2_len_0 {
				v2 = pos1 - verts[3].data.position();
			} else {
				v1 = verts[3].data.position() - pos0;
			}
		}

		v2.cross(v1).normalize()
	}

	fn ensure_face_normals(&mut self) -> bool {
		let mut has_quads = false;
		let section_keys = self.sections.keys().cloned().collect::<Vec<_>>();
		for section_idx in section_keys {
			let face_count = self
				.sections
				.get(&section_idx)
				.map(|section| section.faces.len())
				.unwrap_or(0);
			for face_idx in 0..face_count {
				let needs_normal = {
					let section = self.sections.get(&section_idx).unwrap();
					let face = &section.faces[face_idx];
					if !has_quads && face.vertices.len() > 3 {
						has_quads = true;
					}
					face.face_normal.is_none()
				};

				if needs_normal {
					let normal =
						Some(self.calculate_face_normal(section_index(section_idx, face_idx)));
					self.sections.get_mut(&section_idx).unwrap().faces[face_idx].face_normal =
						normal;
				}
			}
		}
		has_quads
	}

	fn has_quads(&self) -> bool {
		self.sections
			.values()
			.any(|section| section.faces.iter().any(|face| face.vertices.len() > 3))
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
				for section in self.sections.values() {
					for face in section.faces.iter() {
						let index_offset = vertex_count;
						for vertex in &face.vertices {
							buffer.extend(bytemuck::bytes_of(&vertex.data));
							vertex_count += 1;
						}
						if has_quads {
							let face_indices = (0..face.vertices.len())
								.map(|i| i + index_offset)
								.collect::<Vec<_>>();
							fill_face_index_buffer(&mut indices, &face_indices);
						}
					}
				}
			}

			MeshBufferType::FaceVerticesWithVertexNormals => {
				let has_quads = self.ensure_face_normals();
				for (sec_idx, section) in self.sections.iter() {
					for face in section.faces.iter() {
						let index_offset = vertex_count;
						for vertex in &face.vertices {
							let normal = self.calculate_vertex_normal(section_index(
								*sec_idx,
								vertex.position_index,
							));
							buffer.extend(bytemuck::bytes_of(&vertex.data));
							buffer.extend(bytemuck::bytes_of(&normal));
							vertex_count += 1;
						}
						if has_quads {
							let face_indices = (0..face.vertices.len())
								.map(|i| i + index_offset)
								.collect::<Vec<_>>();
							fill_face_index_buffer(&mut indices, &face_indices);
						}
					}
				}
			}

			MeshBufferType::FaceVerticesWithFaceNormals => {
				let has_quads = self.ensure_face_normals();
				for section in self.sections.values() {
					for face in section.faces.iter() {
						let index_offset = vertex_count;
						let normal = face.face_normal.unwrap();
						for vertex in &face.vertices {
							buffer.extend(bytemuck::bytes_of(&vertex.data));
							buffer.extend(bytemuck::bytes_of(&normal));
							vertex_count += 1;
						}
						if has_quads {
							let face_indices = (0..face.vertices.len())
								.map(|i| i + index_offset)
								.collect::<Vec<_>>();
							fill_face_index_buffer(&mut indices, &face_indices);
						}
					}
				}
			}

			MeshBufferType::CompactVertices => {
				let mut index_offset = 0;
				for section in self.sections.values() {
					for vertex_idx in 0..section.positions.len() {
						let data = section.position_face_data(vertex_idx);
						buffer.extend(bytemuck::bytes_of(data));
						vertex_count += 1;
					}

					for face in section.faces.iter() {
						let face_indices = face
							.vertices
							.iter()
							.map(|v| v.position_index + index_offset)
							.collect::<Vec<_>>();
						fill_face_index_buffer(&mut indices, &face_indices);
					}

					index_offset += section.positions.len();
				}
			}

			MeshBufferType::CompactVerticesWithNormal => {
				self.ensure_face_normals();
				let mut index_offset = 0;
				for (sec_idx, section) in self.sections.iter() {
					for vertex_idx in 0..section.positions.len() {
						let data = section.position_face_data(vertex_idx);
						let normal =
							self.calculate_vertex_normal(section_index(*sec_idx, vertex_idx));
						buffer.extend(bytemuck::bytes_of(data));
						buffer.extend(bytemuck::bytes_of(&normal));
						vertex_count += 1;
					}

					for face in section.faces.iter() {
						let face_indices = face
							.vertices
							.iter()
							.map(|v| v.position_index + index_offset)
							.collect::<Vec<_>>();
						fill_face_index_buffer(&mut indices, &face_indices);
					}

					index_offset += section.positions.len();
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
	V: WebglVertexData + Position3D + Clone,
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
