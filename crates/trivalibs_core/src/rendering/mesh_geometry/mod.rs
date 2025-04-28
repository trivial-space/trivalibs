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
		BufferedGeometry,
	},
	utils::default,
};
use glam::Vec3;
use std::collections::{BTreeMap, HashMap};

pub mod utils;

#[derive(Debug)]
pub struct Face<V> {
	pub vertices: Vec<usize>,
	pub face_normal: Option<Vec3>,
	pub data: Option<V>,
	pub section: usize,
}

impl<V> Face<V> {
	fn face3(
		v1_idx: usize,
		v2_idx: usize,
		v3_idx: usize,
		normal: Option<Vec3>,
		data: Option<V>,
		section: usize,
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
			section,
		}
	}

	fn face4(
		v1_idx: usize,
		v2_idx: usize,
		v3_idx: usize,
		v4_idx: usize,
		normal: Option<Vec3>,
		data: Option<V>,
		section: usize,
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
			section,
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

pub struct MeshVertex<V>
where
	V: Overridable + Position3D,
{
	pub data: V,
	pub faces: Vec<usize>,
	pub vertex_normal: Option<Vec3>,
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

pub struct MeshSection<V>
where
	V: Overridable + Position3D,
{
	pub vertices: Vec<MeshVertex<V>>,
	faces: Vec<Face<V>>,
	next_index: usize,
	vertex_indices: HashMap<VertIdx3f, usize>,
	section_index: usize,
}

impl<V> MeshSection<V>
where
	V: Overridable + Position3D + Clone,
{
	pub fn new(section_index: usize) -> Self {
		Self {
			vertices: vec![],
			faces: vec![],
			next_index: 0,
			vertex_indices: HashMap::new(),
			section_index,
		}
	}

	fn get_vertex_index(&mut self, pos: Vec3) -> usize {
		if let Some(idx) = self.vertex_indices.get(&pos.into()) {
			*idx
		} else {
			let idx = self.next_index;
			self.vertex_indices.insert(pos.into(), idx);
			self.next_index = idx + 1;
			idx
		}
	}

	fn add_vertex(&mut self, vertex_idx: usize, face_idx: usize, data: V) {
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
		let v = &mut vertices[vertex_idx];
		v.faces.push(face_idx);
	}

	pub fn add_face3_data(&mut self, verts: [V; 3], data: FaceDataProps<V>) {
		let [v1, v2, v3] = verts;
		let v1_idx = self.get_vertex_index(v1.position());
		let v2_idx = self.get_vertex_index(v2.position());
		let v3_idx = self.get_vertex_index(v3.position());

		let face_idx = self.faces.len();

		let face = Face::face3(
			v1_idx,
			v2_idx,
			v3_idx,
			data.normal,
			data.data,
			self.section_index,
		);
		self.faces.push(face);

		self.add_vertex(v1_idx, face_idx, v1);
		self.add_vertex(v2_idx, face_idx, v2);
		self.add_vertex(v3_idx, face_idx, v3);
	}

	pub fn add_face4_data(&mut self, verts: [V; 4], data: FaceDataProps<V>) {
		let [v1, v2, v3, v4] = verts;
		let v1_idx = self.get_vertex_index(v1.position());
		let v2_idx = self.get_vertex_index(v2.position());
		let v3_idx = self.get_vertex_index(v3.position());
		let v4_idx = self.get_vertex_index(v4.position());

		let face_idx = self.faces.len();

		let face = Face::face4(
			v1_idx,
			v2_idx,
			v3_idx,
			v4_idx,
			data.normal,
			data.data,
			self.section_index,
		);
		self.faces.push(face);

		self.add_vertex(v1_idx, face_idx, v1);
		self.add_vertex(v2_idx, face_idx, v2);
		self.add_vertex(v3_idx, face_idx, v3);
		self.add_vertex(v4_idx, face_idx, v4);
	}

	pub fn face_vertices(&self, face: &Face<V>) -> Vec<V> {
		face.vertices
			.iter()
			.map(|i| self.vertices[*i].data.clone())
			.collect::<Vec<_>>()
	}

	pub fn remove_face(&mut self, face_idx: usize) {
		let vertices: &mut Vec<MeshVertex<V>> = &mut self.vertices;

		let face = self.faces.swap_remove(face_idx);

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

		let len = self.faces.len();
		if face_idx != len {
			let new_face = &self.faces[face_idx];
			for v in &new_face.vertices {
				let vert = &mut vertices[*v];
				let v_f_idx = vert.faces.iter().position(|i| *i == len).unwrap();
				vert.faces[v_f_idx] = face_idx;
			}
		}
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
	V: Overridable + Position3D,
{
	pub sections: BTreeMap<usize, MeshSection<V>>,
}

impl<V> MeshGeometry<V>
where
	V: Overridable + Position3D + Clone,
{
	pub fn new() -> Self {
		Self {
			sections: BTreeMap::new(),
		}
	}

	pub fn vertex_count(&self) -> usize {
		let mut len = 0;
		for section in self.sections.values() {
			len += section.vertices.len();
		}
		len
	}

	pub fn face_count(&self) -> usize {
		let mut len = 0;
		for section in self.sections.values() {
			len += section.faces.len();
		}
		len
	}

	pub fn default_section(&self) -> &MeshSection<V> {
		self.sections.get(&DEFAULT_MESH_SECTION).unwrap()
	}

	pub fn get_or_create_section(&mut self, section_idx: usize) -> &mut MeshSection<V> {
		if !self.sections.contains_key(&section_idx) {
			self.sections
				.insert(section_idx, MeshSection::new(section_idx));
		}
		self.sections.get_mut(&section_idx).unwrap()
	}

	pub fn add_face3_data(&mut self, verts: [V; 3], data: FaceDataProps<V>) {
		let section = self.get_or_create_section(data.section.unwrap_or(DEFAULT_MESH_SECTION));
		section.add_face3_data(verts, data);
	}

	pub fn add_face4_data(&mut self, verts: [V; 4], data: FaceDataProps<V>) {
		let section = self.get_or_create_section(data.section.unwrap_or(DEFAULT_MESH_SECTION));
		section.add_face4_data(verts, data);
	}

	pub fn add_face3(&mut self, verts: [V; 3]) {
		self.add_face3_data(verts, default())
	}

	pub fn add_face4(&mut self, verts: [V; 4]) {
		self.add_face4_data(verts, default())
	}

	pub fn add_grid_ccw_quads_data<A: CoordOpsFn>(
		&mut self,
		grid: &Grid<V, A>,
		data: FaceDataProps<V>,
	) {
		for quad in grid.to_ccw_quads() {
			self.add_face4_data(quad, data.clone());
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
			self.add_face4_data(quad, data.clone());
		}
	}

	pub fn add_grid_cw_quads<A: CoordOpsFn>(&mut self, grid: &Grid<V, A>) {
		self.add_grid_cw_quads_data(grid, default())
	}

	pub fn vertex<T: Into<SectionIndex>>(&self, into_idx: T) -> &MeshVertex<V> {
		let i: SectionIndex = into_idx.into();
		&self.sections.get(&i.section).unwrap().vertices[i.index]
	}

	pub fn get_vertex_index(&self, pos: Vec3) -> Vec<SectionIndex> {
		let mut indices = vec![];
		for (section_idx, section) in self.sections.iter() {
			if let Some(idx) = section.vertex_indices.get(&pos.into()) {
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

	pub fn face_vertices(&self, face: &Face<V>) -> Vec<V> {
		self.sections
			.get(&face.section)
			.unwrap()
			.face_vertices(face)
	}

	pub fn remove_face<T: Into<SectionIndex>>(&mut self, into_idx: T) {
		let face_idx: SectionIndex = into_idx.into();
		self.sections
			.get_mut(&face_idx.section)
			.unwrap()
			.remove_face(face_idx.index);
	}

	pub fn set_vertex<I: Into<SectionIndex>>(&mut self, into_idx: I, data: V) {
		let idx: SectionIndex = into_idx.into();
		let section = self.get_or_create_section(idx.section);
		if let Some(vertex) = section.vertices.get_mut(idx.index) {
			vertex.data = data
		}
	}

	fn calculate_vertex_normal<I: Into<SectionIndex>>(&self, into_idx: I) -> Vec3 {
		let idx: SectionIndex = into_idx.into();
		let section = self.sections.get(&idx.section).unwrap();
		let vert = &section.vertices[idx.index];
		let mut normal = Vec3::ZERO;
		for face_idx in vert.faces.iter() {
			let face = &section.faces[*face_idx];
			normal += face.face_normal.unwrap();
		}
		normal.normalize_or_zero()
	}

	fn calculate_face_normal<I: Into<SectionIndex>>(&self, into_idx: I) -> Vec3 {
		let idx: SectionIndex = into_idx.into();
		let section = self.sections.get(&idx.section).unwrap();

		let face = &section.faces[idx.index];

		let verts = &face.vertices;
		let pos0 = section.vertices[verts[0]].data.position();
		let pos1 = section.vertices[verts[1]].data.position();
		let pos2 = section.vertices[verts[2]].data.position();

		let mut v1 = pos2 - pos0;
		let mut v2 = pos1 - pos0;

		let v1_len = v1.length();
		let v2_len = v2.length();
		let v1_len_0 = v1_len < 0.0001;
		let v2_len_0 = v2_len < 0.0001;
		let v3_len_0 = (v2 / v2_len).dot(v1 / v1_len).abs() > 0.9999;

		if (v1_len_0 || v2_len_0 || v3_len_0) && section.vertices.len() > 3 {
			if v2_len_0 {
				v2 = pos1 - section.vertices[verts[3]].data.position();
			} else {
				v1 = section.vertices[verts[3]].data.position() - pos0;
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
		let section_keys = self.sections.keys().cloned().collect::<Vec<_>>();
		for section_idx in section_keys {
			for face_idx in 0..self.sections.get_mut(&section_idx).unwrap().faces.len() {
				let face = &self.sections.get_mut(&section_idx).unwrap().faces[face_idx];
				if only_triangles && face.vertices.len() > 3 {
					only_triangles = false;
				}
				if face.face_normal.is_none() {
					let normal =
						Some(self.calculate_face_normal(section_index(section_idx, face_idx)));
					let face = &mut self.sections.get_mut(&section_idx).unwrap().faces[face_idx];
					face.face_normal = normal;
				}
			}
		}
		!only_triangles
	}
}

impl<V> MeshGeometry<V>
where
	V: Overridable + Position3D + Clone + bytemuck::Pod,
{
	pub fn to_buffered_geometry_by_type(&mut self, geom_type: MeshBufferType) -> BufferedGeometry {
		let mut buffer = vec![];
		let mut indices = vec![];
		let mut vertex_count = 0;

		match geom_type {
			MeshBufferType::NoNormals => {
				for section in self.sections.values() {
					for vertex in section.vertices.iter() {
						buffer.extend(bytemuck::bytes_of(&vertex.data));
						vertex_count += 1;
					}

					for face in section.faces.iter() {
						fill_face_index_buffer(&mut indices, &face.vertices);
					}
				}
			}

			MeshBufferType::VertexNormals => {
				self.ensure_face_normals();

				let mut idx_offset = 0;

				for (sec_idx, section) in self.sections.iter() {
					for (v_idx, vertex) in section.vertices.iter().enumerate() {
						let normal = self.calculate_vertex_normal(section_index(*sec_idx, v_idx));

						buffer.extend(bytemuck::bytes_of(&vertex.data));
						buffer.extend(bytemuck::bytes_of(&normal));
						vertex_count += 1;
					}

					for face in section.faces.iter() {
						let vert_indices = face
							.vertices
							.iter()
							.map(|v| v + idx_offset)
							.collect::<Vec<_>>();

						fill_face_index_buffer(&mut indices, &vert_indices);
					}

					idx_offset += section.vertices.len();
				}
			}

			MeshBufferType::VertexNormalFaceData => {
				let has_quads = self.ensure_face_normals();

				for (sec_idx, section) in self.sections.iter() {
					for face in section.faces.iter() {
						let index_offset = vertex_count;

						for v in &face.vertices {
							let vertex = &section.vertices[*v];
							let normal = self.calculate_vertex_normal(section_index(*sec_idx, *v));
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

				for section in self.sections.values() {
					for face in section.faces.iter() {
						let index_offset = vertex_count;
						let normal = face.face_normal.unwrap();

						for v in &face.vertices {
							let mut data = section.vertices[*v].data;
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
	V: WebglVertexData + Overridable + Position3D,
{
	pub fn to_webgl_buffered_geometry_by_type(
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
