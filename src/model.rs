use std::path::Path;

use bytemuck::Pod;
use bytemuck::Zeroable;
use tobj::LoadOptions;

use crate::result::Result;
use crate::Loadable;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

#[derive(Default)]
pub struct Model {
    pub meshes: Vec<Mesh>,
}

impl Loadable for Model {
    fn load<P: AsRef<Path>>(path: P) -> Result<Model> {
        let (models, _materials) =
            tobj::load_obj(path.as_ref(), &LoadOptions { triangulate: true, single_index: true, ..Default::default() })?;

        let mut meshes = vec![];
        for model in models {
            let mut vertices = vec![];
            for i in 0..model.mesh.positions.len() / 3 {
                vertices.push(Vertex {
                    position: [
                        model.mesh.positions[i * 3],
                        model.mesh.positions[i * 3 + 1],
                        model.mesh.positions[i * 3 + 2],
                    ],
                    tex_coords: [model.mesh.texcoords[i * 2], model.mesh.texcoords[i * 2 + 1]],
                    normal: [model.mesh.normals[i * 3], model.mesh.normals[i * 3 + 1], model.mesh.normals[i * 3 + 2]],
                });
            }

            meshes.push(Mesh { vertices, indices: model.mesh.indices });
        }

        Ok(Model { meshes })
    }
}
