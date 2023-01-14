use std::{path::Path, rc::Rc};

use eyre::{eyre, Result};
use glam::{Mat4, Quat, Vec3};
use gltf::scene::Transform as GTransform;

mod mesh;

use crate::ogl::texture::GlTexture;

pub use self::mesh::{Mesh, Primitive};

/// Image and vertex data of the asset.
pub struct DataBundle {
    /// Vertex data
    buffers: Vec<gltf::buffer::Data>,
    /// Texture data
    images: Vec<gltf::image::Data>,
    /// To keep track if which textures were already sent to the GPU
    pub gl_textures: Vec<Option<Rc<GlTexture>>>,
}

impl DataBundle {
    fn new(buffers: Vec<gltf::buffer::Data>, images: Vec<gltf::image::Data>) -> Self {
        Self {
            buffers,
            gl_textures: vec![Option::None; images.len()],
            images,
        }
    }
}

/// This represents a gltf sacene and contains necessary data for rendering.
pub struct Scene {
    /// An artifical root node
    pub root: Node,
    /// Name of the scene
    pub name: String,
    /// Model transforms of the whole object
    pub transform: Mat4,
}

impl Scene {
    /// Load the scene from a path to a gltf file
    pub fn from_gltf(path: &str) -> Result<Scene> {
        let (gltf, buffers, images) = gltf::import(path)?;
        let name = Path::new(path)
            .file_name()
            .map(|osstr| osstr.to_string_lossy().to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let mut bundle = DataBundle::new(buffers, images);

        if gltf.scenes().len() != 1 {
            return Err(eyre!("GLTF file contains more than 1 scene"));
        }
        let scene = gltf.scenes().next().unwrap();

        let mut id = 1;
        let mut nodes = Vec::new();
        for node in scene.nodes() {
            let node = Node::from_gltf(&node, &mut bundle, &mut id, &scene)?;
            id += 1;
            nodes.push(node);
        }

        let root = Node {
            index: usize::MAX,
            name: scene.name().unwrap_or("Root").to_string(),
            children: nodes,
            mesh: None,
            transform: Mat4::IDENTITY,
        };

        Ok(Scene {
            root,
            name,
            transform: Mat4::IDENTITY,
        })
    }
}

/// A Node represents a subset of a gltf scene
/// Nodes form a tree hierarchy
pub struct Node {
    /// The same index as in the gltf file
    pub index: usize,
    /// Name of the node
    pub name: String,
    /// Children nodes
    pub children: Vec<Node>,
    /// Optional mesh data of the node (can contain multiple primitives)
    pub mesh: Option<Mesh>,
    /// Transform of the node in the hierarchy
    pub transform: Mat4,
}

impl Node {
    /// Crate a node from a gltf::Node structure
    fn from_gltf(
        node: &gltf::Node,
        bundle: &mut DataBundle,
        id: &mut u32,
        scene: &gltf::Scene,
    ) -> Result<Self> {
        let mut children = Vec::new();

        let name = node.name().unwrap_or(&format!("Node-{id}")).to_string();

        for child_node in node.children() {
            *id += 1;
            let node = Node::from_gltf(&child_node, bundle, id, scene)?;
            children.push(node);
        }

        let mesh = match node.mesh() {
            Some(m) => Some(Mesh::from_gltf(&m, bundle)?),
            None => None,
        };

        let transform = match node.transform() {
            GTransform::Matrix { matrix } => Mat4::from_cols_array_2d(&matrix),
            GTransform::Decomposed {
                translation,
                rotation,
                scale,
            } => {
                Mat4::from_translation(Vec3::from(translation))
                    * Mat4::from_quat(Quat::from_xyzw(
                        rotation[0],
                        rotation[1],
                        rotation[2],
                        rotation[3],
                    ))
                    * Mat4::from_scale(Vec3::from(scale))
            }
        };

        Ok(Self {
            index: node.index(),
            children,
            mesh,
            transform,
            name,
        })
    }
}
