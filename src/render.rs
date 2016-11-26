use std::path::PathBuf;
use std::collections::HashMap;

use glium::{VertexBuffer, Program};
use glium::Display;
use glium::index::{IndexBuffer, PrimitiveType};

use nalgebra::Eye;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}

pub fn vert(x: f32, y: f32) -> Vertex {
    Vertex {
        position: [x, y],
        tex_coords: [0., 0.],
    }
}

pub fn vertex(x: f32, y: f32, u: f32, v: f32) -> Vertex {
    Vertex {
        position: [x, y],
        tex_coords: [u, v],
    }
}

pub struct Model {
    pub vertices: VertexBuffer<Vertex>,
    pub indices: IndexBuffer<u32>,
}

impl Model {
    pub fn new(vertices: VertexBuffer<Vertex>, indices: IndexBuffer<u32>) -> Model {
        Model {
            vertices: vertices,
            indices: indices,
        }
    }
}

implement_vertex!(Vertex, position, tex_coords);

pub struct RenderContext<'a> {
    pub fonts: Fonts<'a>,
    pub cam: Mat,
    pub programs: HashMap<String, Program>,
    pub models: HashMap<String, Model>,
}
