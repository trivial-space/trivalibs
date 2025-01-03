use crate::{
	data::{
		grid::{CoordOpsFn, Grid},
		vertex_index::VertIdx3f,
		Overridable, Position3D,
	},
	rendering::{
		webgl_buffered_geometry::{
			create_buffered_geometry_layout, RenderingPrimitive, VertexFormat, VertexType,
			WebglBufferedGeometry, WebglVertexData,
		},
		RenderableBuffer,
	},
	utils::default,
};
use glam::Vec3;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug)]
pub struct Face<V> {
	pub vertices: Vec<usize>,
	pub face_normal: Option<Vec3>,
	pub data: Option<V>,
}

impl<V> Face<V> {
	fn face3(
		v1_idx: usize,
		v2_idx: usize,
		v3_idx: usize,
		normal: Option<Vec3>,
		data: Option<V>,
	) -> Face<V> {
		if v1_idx == v2_idx || v1_idx == v3_idx || v2_idx == v3_idx {
			panic!("Face must have 3 unique vertices");
		}
		let mut vertices = Vec::with_capacity(3);
		vertices.push(v1_idx);
		vertices.push(v2_idx);
		vertices.push(v3_idx);

		Face {
			face_normal: normal,
			vertices,
			data,
		}
	}

	fn face4(
		v1_idx: usize,
		v2_idx: usize,
		v3_idx: usize,
		v4_idx: usize,
		normal: Option<Vec3>,
		data: Option<V>,
	) -> Face<V> {
		if v1_idx == v2_idx
			|| v1_idx == v3_idx
			|| v1_idx == v4_idx
			|| v2_idx == v3_idx
			|| v2_idx == v4_idx
			|| v3_idx == v4_idx
		{
			panic!("Face must have 4 unique vertices");
		}
		let mut vertices = Vec::with_capacity(4);
		vertices.push(v1_idx);
		vertices.push(v2_idx);
		vertices.push(v3_idx);
		vertices.push(v4_idx);

		Face {
			face_normal: normal,
			vertices,
			data,
		}
	}
}

#[derive(PartialEq)]
pub enum MeshBufferType {
	NoNormals,
	VertexNormals,
	VertexNormalFaceData,
	FaceNormals,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct SectionIndex {
	pub section: usize,
	pub index: usize,
}

impl From<usize> for SectionIndex {
	fn from(index: usize) -> Self {
		Self { section: 0, index }
	}
}

pub struct MeshVertex<V>
where
	V: Overridable + Position3D,
{
	pub data: V,
	pub faces: Vec<SectionIndex>,
	pub vertex_normal: Option<Vec3>,
}

impl<V> MeshVertex<V>
where
	V: Overridable + Position3D,
{
	fn section_faces(&self, section: usize) -> Vec<usize> {
		self.faces
			.iter()
			.filter(|section_index| section_index.section == section)
			.map(|section_index| section_index.index)
			.collect()
	}
}

pub struct MeshGeometry<V>
where
	V: Overridable + Position3D,
{
	pub vertices: Vec<MeshVertex<V>>,
	faces: Vec<Vec<Face<V>>>,
	next_index: usize,
	vertex_indices: HashMap<VertIdx3f, usize>,
}

#[derive(Debug, Copy, Clone)]
pub struct FaceDataProps<V>
where
	V: Overridable + Position3D,
{
	pub normal: Option<Vec3>,
	pub data: Option<V>,
	pub section: Option<usize>,
}

impl<V> Default for FaceDataProps<V>
where
	V: Overridable + Position3D,
{
	fn default() -> Self {
		Self {
			normal: None,
			data: None,
			section: None,
		}
	}
}

impl<V> FaceDataProps<V>
where
	V: Overridable + Position3D,
{
	pub fn with_normal(&mut self, normal: Vec3) -> &mut Self {
		self.normal = Some(normal);
		self
	}
	pub fn with_data(&mut self, data: V) -> &mut Self {
		self.data = Some(data);
		self
	}
	pub fn with_section(&mut self, section: usize) -> &mut Self {
		self.section = Some(section);
		self
	}
}

pub fn face_normal<V>(normal: Vec3) -> FaceDataProps<V>
where
	V: Overridable + Position3D,
{
	FaceDataProps {
		normal: Some(normal),
		data: None,
		section: None,
	}
}

pub fn face_data<V>(data: V) -> FaceDataProps<V>
where
	V: Overridable + Position3D,
{
	FaceDataProps {
		normal: None,
		data: Some(data),
		section: None,
	}
}

pub fn face_section<V>(section: usize) -> FaceDataProps<V>
where
	V: Overridable + Position3D,
{
	FaceDataProps {
		normal: None,
		data: None,
		section: Some(section),
	}
}

impl<V> MeshGeometry<V>
where
	V: Overridable + Position3D + Clone,
{
	pub fn new() -> Self {
		Self {
			vertices: vec![],
			faces: Vec::with_capacity(1),
			next_index: 0,
			vertex_indices: HashMap::new(),
		}
	}

	pub fn add_face3_data(&mut self, v1: V, v2: V, v3: V, data: FaceDataProps<V>) {
		let v1_idx = self.get_vertex_index(v1.position());
		let v2_idx = self.get_vertex_index(v2.position());
		let v3_idx = self.get_vertex_index(v3.position());

		let section = data.section.unwrap_or(0);
		let faces = if let Some(faces) = self.faces.get_mut(section) {
			faces
		} else {
			let faces = Vec::new();
			self.faces.push(faces);
			self.faces.last_mut().unwrap()
		};

		let face_idx = SectionIndex {
			index: faces.len(),
			section,
		};

		let face = Face::face3(v1_idx, v2_idx, v3_idx, data.normal, data.data);
		faces.push(face);

		self.add_vertex(v1_idx, face_idx, v1);
		self.add_vertex(v2_idx, face_idx, v2);
		self.add_vertex(v3_idx, face_idx, v3);
	}

	pub fn add_face4_data(&mut self, v1: V, v2: V, v3: V, v4: V, data: FaceDataProps<V>) {
		let v1_idx = self.get_vertex_index(v1.position());
		let v2_idx = self.get_vertex_index(v2.position());
		let v3_idx = self.get_vertex_index(v3.position());
		let v4_idx = self.get_vertex_index(v4.position());

		let section = data.section.unwrap_or(0);
		let faces = if let Some(faces) = self.faces.get_mut(section) {
			faces
		} else {
			let faces = Vec::new();
			self.faces.push(faces);
			self.faces.last_mut().unwrap()
		};

		let face_idx = SectionIndex {
			index: faces.len(),
			section,
		};

		let face = Face::face4(v1_idx, v2_idx, v3_idx, v4_idx, data.normal, data.data);
		faces.push(face);

		self.add_vertex(v1_idx, face_idx, v1);
		self.add_vertex(v2_idx, face_idx, v2);
		self.add_vertex(v3_idx, face_idx, v3);
		self.add_vertex(v4_idx, face_idx, v4);
	}

	pub fn add_face3(&mut self, v1: V, v2: V, v3: V) {
		self.add_face3_data(v1, v2, v3, default())
	}

	pub fn add_face4(&mut self, v1: V, v2: V, v3: V, v4: V) {
		self.add_face4_data(v1, v2, v3, v4, default())
	}

	pub fn add_grid_ccw_quads_data<A: CoordOpsFn>(
		&mut self,
		grid: &Grid<V, A>,
		data: FaceDataProps<V>,
	) {
		for quad in grid.to_ccw_quads() {
			self.add_face4_data(
				quad[0].clone(),
				quad[1].clone(),
				quad[2].clone(),
				quad[3].clone(),
				data.clone(),
			);
		}
	}

	pub fn add_grid_ccw_quads<A: CoordOpsFn>(&mut self, grid: &Grid<V, A>) {
		self.add_grid_ccw_quads_data(grid, default())
	}

	pub fn add_grid_cw_quads_data<A: CoordOpsFn>(
		&mut self,
		grid: &Grid<V, A>,
		data: FaceDataProps<V>,
	) {
		for quad in grid.to_cw_quads() {
			self.add_face4_data(
				quad[0].clone(),
				quad[1].clone(),
				quad[2].clone(),
				quad[3].clone(),
				data.clone(),
			);
		}
	}

	pub fn add_grid_cw_quads<A: CoordOpsFn>(&mut self, grid: &Grid<V, A>) {
		self.add_grid_cw_quads_data(grid, default())
	}

	pub fn vertex(&self, i: usize) -> &MeshVertex<V> {
		&self.vertices[i]
	}

	pub fn face<T: Into<SectionIndex>>(&self, into_idx: T) -> &Face<V> {
		let i: SectionIndex = into_idx.into();
		&self.faces.get(i.section).unwrap()[i.index]
	}

	pub fn get_vertex_index(&mut self, pos: Vec3) -> usize {
		if let Some(idx) = self.vertex_indices.get(&pos.into()) {
			*idx
		} else {
			let idx = self.next_index;
			self.vertex_indices.insert(pos.into(), idx);
			self.next_index = idx + 1;
			idx
		}
	}

	pub fn remove_face<T: Into<SectionIndex>>(&mut self, face_idx: T) {
		let face_idx: SectionIndex = face_idx.into();
		let faces: &mut Vec<Face<V>> = self.faces.get_mut(face_idx.section).unwrap();
		let vertices: &mut Vec<MeshVertex<V>> = &mut self.vertices;

		let face = faces.swap_remove(face_idx.index);

		face.vertices.into_iter().for_each(|i| {
			let vert = &mut vertices[i];
			let new_faces = vert
				.faces
				.iter()
				.map(|j| *j)
				.filter(|j| *j != face_idx)
				.collect();
			vert.faces = new_faces;
		});

		let len = faces.len();
		if face_idx.index != len {
			let new_face = &faces[face_idx.index];
			for v in &new_face.vertices {
				let vert = &mut vertices[*v];
				let v_f_idx = vert
					.faces
					.iter()
					.position(|i| {
						*i == SectionIndex {
							index: len,
							section: face_idx.section,
						}
					})
					.unwrap();
				vert.faces[v_f_idx] = face_idx;
			}
		}
	}

	pub fn set_vertex(&mut self, vertex_idx: usize, data: V) {
		if let Some(vertex) = self.vertices.get_mut(vertex_idx) {
			vertex.data = data
		}
	}

	fn add_vertex(&mut self, vertex_idx: usize, face_idx: SectionIndex, data: V) {
		let vertices = &mut self.vertices;
		if let Some(v) = vertices.get_mut(vertex_idx) {
			v.data = data;
		} else {
			vertices.push(MeshVertex {
				data,
				faces: Vec::with_capacity(8),
				vertex_normal: None,
			});
		}
		Self::add_vertex_face(vertices, vertex_idx, face_idx);
	}

	fn add_vertex_face(
		vertices: &mut Vec<MeshVertex<V>>,
		vertex_idx: usize,
		face_idx: SectionIndex,
	) {
		let v = &mut vertices[vertex_idx];
		v.faces.push(face_idx);
	}

	fn calculate_vertex_normal(&self, section: usize, vert_index: usize) -> Vec3 {
		let vert = &self.vertices[vert_index];
		let face_indices = vert.section_faces(section);
		let mut normal = Vec3::ZERO;
		for face_idx in face_indices {
			let face = &self.faces[section][face_idx];
			normal += face.face_normal.unwrap();
		}
		normal.normalize_or_zero()
	}

	fn calculate_face_normal(&self, section: usize, face_index: usize) -> Vec3 {
		let face = &self.faces[section][face_index];

		let verts = &face.vertices;
		let pos0 = self.vertices[verts[0]].data.position();
		let pos1 = self.vertices[verts[1]].data.position();
		let pos2 = self.vertices[verts[2]].data.position();

		let mut v1 = pos2 - pos0;
		let mut v2 = pos1 - pos0;

		let v1_len = v1.length();
		let v2_len = v2.length();
		let v1_len_0 = v1_len < 0.0001;
		let v2_len_0 = v2_len < 0.0001;
		let v3_len_0 = (v2 / v2_len).dot(v1 / v1_len).abs() > 0.9999;

		if (v1_len_0 || v2_len_0 || v3_len_0) && self.vertices.len() > 3 {
			if v2_len_0 {
				v2 = pos1 - self.vertices[verts[3]].data.position();
			} else {
				v1 = self.vertices[verts[3]].data.position() - pos0;
			}
		}

		let normal = v2.cross(v1).normalize();
		normal
	}

	/// Calculates face normals for all faces that don't have them
	///
	/// # Returns
	///
	/// Returns true if there are any faces that have more than 3 vertices
	fn ensure_face_normals(&mut self) -> bool {
		let mut only_triangles = true;
		for i in 0..self.faces.len() {
			for j in 0..self.faces[i].len() {
				if only_triangles && self.faces[i][j].vertices.len() > 3 {
					only_triangles = false;
				}
				if self.faces[i][j].face_normal.is_none() {
					self.faces[i][j].face_normal = Some(self.calculate_face_normal(i, j));
				}
			}
		}
		!only_triangles
	}
}

impl<V> MeshGeometry<V>
where
	V: Overridable + Position3D + Copy + bytemuck::Pod,
{
	pub fn to_renderable_buffer_by_type(&mut self, geom_type: MeshBufferType) -> RenderableBuffer {
		let mut buffer = vec![];
		let mut indices = vec![];
		let mut vertex_count = 0;

		match geom_type {
			MeshBufferType::NoNormals => {
				for vertex in self.vertices.iter() {
					buffer.extend(bytemuck::bytes_of(&vertex.data));
					vertex_count += 1;
				}

				for faces in self.faces.iter() {
					for face in faces {
						fill_face_index_buffer(&mut indices, &face.vertices);
					}
				}
			}

			MeshBufferType::VertexNormals => {
				self.ensure_face_normals();

				let mut section_vert_indices = BTreeMap::<usize, BTreeMap<usize, usize>>::new();
				let mut section_vertices = BTreeMap::<usize, Vec<usize>>::new();

				for (i, vertex) in self.vertices.iter().enumerate() {
					for face_idx in &vertex.faces {
						let section = face_idx.section;

						if !section_vertices.contains_key(&section) {
							section_vertices.insert(section, vec![]);
						}

						if !section_vert_indices.contains_key(&i) {
							section_vert_indices.insert(i, BTreeMap::new());
						}

						let index = section_vertices.get(&section).unwrap().len();

						section_vert_indices
							.get_mut(&i)
							.unwrap()
							.insert(section, index);

						section_vertices.get_mut(&section).unwrap().push(i);
					}
				}

				let mut idx_offset = 0;

				for (section, faces) in self.faces.iter().enumerate() {
					for v_idx in section_vertices.get(&section).unwrap() {
						let vertex = &self.vertices[*v_idx];

						let normal = self.calculate_vertex_normal(section, *v_idx);

						buffer.extend(bytemuck::bytes_of(&vertex.data));
						buffer.extend(bytemuck::bytes_of(&normal));
						vertex_count += 1;
					}

					for face in faces {
						let vert_indices = face
							.vertices
							.iter()
							.map(|v| {
								section_vert_indices.get(v).unwrap().get(&section).unwrap()
									+ idx_offset
							})
							.collect::<Vec<_>>();

						fill_face_index_buffer(&mut indices, &vert_indices);
					}

					idx_offset += section_vertices.get(&section).unwrap().len();
				}
			}

			MeshBufferType::VertexNormalFaceData => {
				let has_quads = self.ensure_face_normals();

				for (section, faces) in self.faces.iter().enumerate() {
					for face in faces {
						let index_offset = vertex_count;

						for v in &face.vertices {
							let vertex = &self.vertices[*v];
							let normal = self.calculate_vertex_normal(section, *v);
							let mut data = vertex.data;
							if face.data.is_some() {
								data = data.override_with(&face.data.unwrap());
							}
							buffer.extend(bytemuck::bytes_of(&data));
							buffer.extend(bytemuck::bytes_of(&normal));
							vertex_count += 1;
						}

						if has_quads {
							fill_face_index_buffer(
								&mut indices,
								&(0..face.vertices.len())
									.map(|i| i + index_offset)
									.collect::<Vec<_>>(),
							);
						}
					}
				}
			}

			MeshBufferType::FaceNormals => {
				let has_quads = self.ensure_face_normals();

				for faces in self.faces.iter_mut() {
					for face in faces {
						let index_offset = vertex_count;
						let normal = face.face_normal.unwrap();

						for v in &face.vertices {
							let mut data = self.vertices[*v].data;
							if face.data.is_some() {
								data = data.override_with(&face.data.unwrap());
							}
							buffer.extend(bytemuck::bytes_of(&data));
							buffer.extend(bytemuck::bytes_of(&normal));
							vertex_count += 1;
						}

						if has_quads {
							fill_face_index_buffer(
								&mut indices,
								&(0..face.vertices.len())
									.map(|i| i + index_offset)
									.collect::<Vec<_>>(),
							);
						}
					}
				}
			}
		};

		let indices_len = indices.len();

		RenderableBuffer {
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
	V: WebglVertexData + Overridable + Position3D,
{
	pub fn to_buffered_geometry_by_type(
		&mut self,
		geom_type: MeshBufferType,
	) -> WebglBufferedGeometry {
		let mut layout: Vec<VertexType> = V::vertex_layout();

		if geom_type != MeshBufferType::NoNormals {
			layout.push(VertexType {
				name: "normal",
				format: VertexFormat::Float32x3,
			})
		}

		let buffer = self.to_renderable_buffer_by_type(geom_type);

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

impl<V> Position3D for MeshVertex<V>
where
	V: Overridable + Position3D,
{
	fn position(&self) -> Vec3 {
		self.data.position()
	}
}

#[cfg(test)]
mod tests;
