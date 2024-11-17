use super::vertex_index::VertIdx3f;
use crate::{
	data_structures::grid::{CoordOpsFn, Grid},
	rendering::buffered_geometry::{
		create_buffered_geometry_layout, BufferedGeometry, BufferedVertexData,
		OverrideAttributesWith, RenderingPrimitive, VertexFormat, VertexType,
	},
	utils::default,
};
use glam::Vec3;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug)]
pub struct Face<BV>
where
	BV: BufferedVertexData + OverrideAttributesWith,
{
	pub vertices: Vec<usize>,
	pub face_normal: Option<Vec3>,
	pub data: Option<BV>,
}

impl<BV> Face<BV>
where
	BV: BufferedVertexData + OverrideAttributesWith,
{
	fn face3(
		v1_idx: usize,
		v2_idx: usize,
		v3_idx: usize,
		normal: Option<Vec3>,
		data: Option<BV>,
	) -> Face<BV> {
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
		data: Option<BV>,
	) -> Face<BV> {
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

pub trait Position3D: BufferedVertexData {
	fn position(&self) -> Vec3;
}

pub enum MeshBufferedGeometryType {
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

pub struct MeshVertex<BV>
where
	BV: BufferedVertexData + OverrideAttributesWith + Position3D,
{
	pub data: BV,
	pub faces: Vec<SectionIndex>,
	pub vertex_normal: Option<Vec3>,
}

impl<BV> MeshVertex<BV>
where
	BV: BufferedVertexData + OverrideAttributesWith + Position3D,
{
	fn section_faces(&self, section: usize) -> Vec<usize> {
		self.faces
			.iter()
			.filter(|section_index| section_index.section == section)
			.map(|section_index| section_index.index)
			.collect()
	}
}

pub struct MeshGeometry<BV>
where
	BV: BufferedVertexData + OverrideAttributesWith + Position3D,
{
	pub vertices: Vec<MeshVertex<BV>>,
	faces: BTreeMap<usize, Vec<Face<BV>>>,
	next_index: usize,
	vertex_indices: HashMap<VertIdx3f, usize>,
}

#[derive(Debug, Copy, Clone)]
pub struct FaceDataProps<BV>
where
	BV: BufferedVertexData + OverrideAttributesWith + Position3D,
{
	pub normal: Option<Vec3>,
	pub data: Option<BV>,
	pub section: Option<usize>,
}

impl<BV> Default for FaceDataProps<BV>
where
	BV: BufferedVertexData + OverrideAttributesWith + Position3D,
{
	fn default() -> Self {
		Self {
			normal: None,
			data: None,
			section: None,
		}
	}
}

impl<BV> FaceDataProps<BV>
where
	BV: BufferedVertexData + OverrideAttributesWith + Position3D,
{
	pub fn with_normal(&mut self, normal: Vec3) -> &mut Self {
		self.normal = Some(normal);
		self
	}
	pub fn with_data(&mut self, data: BV) -> &mut Self {
		self.data = Some(data);
		self
	}
	pub fn with_section(&mut self, section: usize) -> &mut Self {
		self.section = Some(section);
		self
	}
}

pub fn face_normal<BV>(normal: Vec3) -> FaceDataProps<BV>
where
	BV: BufferedVertexData + OverrideAttributesWith + Position3D,
{
	FaceDataProps {
		normal: Some(normal),
		data: None,
		section: None,
	}
}

pub fn face_data<BV>(data: BV) -> FaceDataProps<BV>
where
	BV: BufferedVertexData + OverrideAttributesWith + Position3D,
{
	FaceDataProps {
		normal: None,
		data: Some(data),
		section: None,
	}
}

pub fn face_section<BV>(section: usize) -> FaceDataProps<BV>
where
	BV: BufferedVertexData + OverrideAttributesWith + Position3D,
{
	FaceDataProps {
		normal: None,
		data: None,
		section: Some(section),
	}
}

impl<BV> MeshGeometry<BV>
where
	BV: BufferedVertexData + OverrideAttributesWith + Position3D,
{
	pub fn new() -> Self {
		Self {
			vertices: vec![],
			faces: BTreeMap::new(),
			next_index: 0,
			vertex_indices: HashMap::new(),
		}
	}

	pub fn add_face3_data(&mut self, v1: BV, v2: BV, v3: BV, data: FaceDataProps<BV>) {
		let v1_idx = self.get_vertex_index(v1.position());
		let v2_idx = self.get_vertex_index(v2.position());
		let v3_idx = self.get_vertex_index(v3.position());

		let section = data.section.unwrap_or(0);
		let faces = self.faces.entry(section).or_insert_with(Vec::new);

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

	pub fn add_face4_data(&mut self, v1: BV, v2: BV, v3: BV, v4: BV, data: FaceDataProps<BV>) {
		let v1_idx = self.get_vertex_index(v1.position());
		let v2_idx = self.get_vertex_index(v2.position());
		let v3_idx = self.get_vertex_index(v3.position());
		let v4_idx = self.get_vertex_index(v4.position());

		let section = data.section.unwrap_or(0);
		let faces = self.faces.entry(section).or_insert_with(Vec::new);

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

	pub fn add_face3(&mut self, v1: BV, v2: BV, v3: BV) {
		self.add_face3_data(v1, v2, v3, default())
	}

	pub fn add_face4(&mut self, v1: BV, v2: BV, v3: BV, v4: BV) {
		self.add_face4_data(v1, v2, v3, v4, default())
	}

	pub fn add_grid_ccw_quads_data<A: CoordOpsFn>(
		&mut self,
		grid: &Grid<BV, A>,
		data: FaceDataProps<BV>,
	) {
		for quad in grid.to_ccw_quads() {
			self.add_face4_data(quad[0], quad[1], quad[2], quad[3], data.clone());
		}
	}

	pub fn add_grid_ccw_quads<A: CoordOpsFn>(&mut self, grid: &Grid<BV, A>) {
		self.add_grid_ccw_quads_data(grid, default())
	}

	pub fn add_grid_cw_quads_data<A: CoordOpsFn>(
		&mut self,
		grid: &Grid<BV, A>,
		data: FaceDataProps<BV>,
	) {
		for quad in grid.to_cw_quads() {
			self.add_face4_data(quad[0], quad[1], quad[2], quad[3], data.clone());
		}
	}

	pub fn add_grid_cw_quads<A: CoordOpsFn>(&mut self, grid: &Grid<BV, A>) {
		self.add_grid_cw_quads_data(grid, default())
	}

	pub fn vertex(&self, i: usize) -> &MeshVertex<BV> {
		&self.vertices[i]
	}

	pub fn face<T: Into<SectionIndex>>(&self, into_idx: T) -> &Face<BV> {
		let i: SectionIndex = into_idx.into();
		&self.faces.get(&i.section).unwrap()[i.index]
	}

	fn triangulate(&mut self) {
		let vertices = &mut self.vertices;
		for (section, faces) in self.faces.iter_mut() {
			let quads = faces
				.iter()
				.enumerate()
				.filter(|(_, f)| f.vertices.len() == 4)
				.map(|(i, f)| (i, f.vertices.clone(), f.face_normal, f.data))
				.rev()
				.collect::<Vec<_>>();

			let section = *section;

			for (i, verts, normal, data) in quads {
				Self::remove_face_internal(faces, vertices, SectionIndex { section, index: i });

				let face_idx1 = SectionIndex {
					section,
					index: faces.len(),
				};

				let f = Face::face3(verts[0], verts[1], verts[2], normal, data);
				faces.push(f);

				Self::add_vertex_face(vertices, verts[0], face_idx1);
				Self::add_vertex_face(vertices, verts[1], face_idx1);
				Self::add_vertex_face(vertices, verts[2], face_idx1);

				let face_idx2 = SectionIndex {
					section,
					index: faces.len(),
				};

				let f = Face::face3(verts[0], verts[2], verts[3], normal, data);
				faces.push(f);

				Self::add_vertex_face(vertices, verts[0], face_idx2);
				Self::add_vertex_face(vertices, verts[2], face_idx2);
				Self::add_vertex_face(vertices, verts[3], face_idx2);
			}
		}
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

	fn remove_face_internal(
		faces: &mut Vec<Face<BV>>,
		vertices: &mut Vec<MeshVertex<BV>>,
		face_idx: SectionIndex,
	) {
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

	pub fn remove_face<T: Into<SectionIndex>>(&mut self, face_idx: T) {
		let face_idx: SectionIndex = face_idx.into();
		Self::remove_face_internal(
			self.faces.get_mut(&face_idx.section).unwrap(),
			&mut self.vertices,
			face_idx,
		)
	}

	pub fn set_vertex(&mut self, vertex_idx: usize, data: BV) {
		if let Some(vertex) = self.vertices.get_mut(vertex_idx) {
			vertex.data = data
		}
	}

	fn generate_face_normals(&mut self) {
		for (_, faces) in self.faces.iter_mut() {
			for face in faces.iter_mut() {
				if face.face_normal.is_none() {
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

					if (v1_len_0 || v2_len_0 || v3_len_0) && face.vertices.len() > 3 {
						if v2_len_0 {
							v2 = pos1 - self.vertices[verts[3]].data.position();
						} else {
							v1 = self.vertices[verts[3]].data.position() - pos0;
						}
					}

					let normal = v2.cross(v1);
					face.face_normal = Some(normal.normalize());
				}
			}
		}
	}

	pub fn to_buffered_geometry_by_type(
		&mut self,
		geom_type: MeshBufferedGeometryType,
	) -> BufferedGeometry {
		let mut buffer = vec![];
		let mut indices = vec![];
		let mut layout: Vec<VertexType> = BV::vertex_layout();

		match geom_type {
			MeshBufferedGeometryType::NoNormals => {
				self.triangulate();

				for vertex in self.vertices.iter() {
					buffer.extend(bytemuck::bytes_of(&vertex.data));
				}

				for (_, faces) in self.faces.iter() {
					for face in faces {
						for v in &face.vertices {
							let i = *v as u32;
							indices.extend(bytemuck::bytes_of(&i))
						}
					}
				}
			}

			MeshBufferedGeometryType::VertexNormals => {
				self.generate_face_normals();
				self.triangulate();

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

				for (section, faces) in self.faces.iter() {
					for v_idx in section_vertices.get(&section).unwrap() {
						let vertex = &self.vertices[*v_idx];

						let normal =
							Self::calculate_vertex_normal(faces, &vertex.section_faces(*section));

						buffer.extend(bytemuck::bytes_of(&vertex.data));
						buffer.extend(bytemuck::bytes_of(&normal));
					}

					for face in faces {
						for v in &face.vertices {
							let section_idx =
								section_vert_indices.get(v).unwrap().get(&section).unwrap();
							let i = section_idx + idx_offset;
							indices.extend(bytemuck::bytes_of(&i))
						}
					}

					idx_offset += section_vertices.get(&section).unwrap().len();
				}

				layout.push(VertexType {
					name: "normal",
					format: VertexFormat::Float32x3,
				})
			}

			MeshBufferedGeometryType::VertexNormalFaceData => {
				self.generate_face_normals();
				self.triangulate();

				for (section, faces) in self.faces.iter() {
					for face in faces {
						for v in &face.vertices {
							let vertex = &mut self.vertices[*v];
							let normal = Self::calculate_vertex_normal(
								faces,
								&vertex.section_faces(*section),
							);
							let mut data = vertex.data;
							if face.data.is_some() {
								data = data.override_with(&face.data.unwrap());
							}
							buffer.extend(bytemuck::bytes_of(&data));
							buffer.extend(bytemuck::bytes_of(&normal));
						}
					}
				}
				layout.push(VertexType {
					name: "normal",
					format: VertexFormat::Float32x3,
				})
			}

			MeshBufferedGeometryType::FaceNormals => {
				self.generate_face_normals();
				self.triangulate();
				for (_, faces) in self.faces.iter() {
					for face in faces {
						let normal = face.face_normal.unwrap();
						for v in &face.vertices {
							let mut data = self.vertices[*v].data;
							if face.data.is_some() {
								data = data.override_with(&face.data.unwrap());
							}
							buffer.extend(bytemuck::bytes_of(&data));
							buffer.extend(bytemuck::bytes_of(&normal));
						}
					}
				}
				layout.push(VertexType {
					name: "normal",
					format: VertexFormat::Float32x3,
				})
			}
		};

		let geom_layout = create_buffered_geometry_layout(layout);

		let buffer_len = buffer.len() as u32;
		let indices_len = indices.len() as u32;

		BufferedGeometry {
			buffer,
			rendering_primitive: RenderingPrimitive::Triangles,
			indices: if indices_len == 0 {
				None
			} else {
				Some(indices)
			},
			vertex_size: geom_layout.vertex_size,
			vertex_count: if indices_len > 0 {
				indices_len / 4 // 4 bytes per u32 index
			} else {
				buffer_len / geom_layout.vertex_size
			},
			vertex_layout: geom_layout.vertex_layout,
		}
	}

	fn add_vertex(&mut self, vertex_idx: usize, face_idx: SectionIndex, data: BV) {
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
		vertices: &mut Vec<MeshVertex<BV>>,
		vertex_idx: usize,
		face_idx: SectionIndex,
	) {
		let v = &mut vertices[vertex_idx];
		v.faces.push(face_idx);
	}

	fn calculate_vertex_normal(faces: &Vec<Face<BV>>, face_indices: &Vec<usize>) -> Vec3 {
		let mut normal = Vec3::ZERO;
		for face_idx in face_indices {
			let face = &faces[*face_idx];
			normal += face.face_normal.unwrap();
		}
		normal.normalize_or_zero()
	}
}

#[cfg(test)]
mod tests;
