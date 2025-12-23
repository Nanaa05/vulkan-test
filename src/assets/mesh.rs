use crate::resources::buffer::Vertex;

pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub fn cube() -> MeshData {
    let vertices = vec![
        // Front
        Vertex {
            pos: [-0.5, -0.5, 0.5],
            color: [1.0, 0.0, 0.0],
        },
        Vertex {
            pos: [0.5, -0.5, 0.5],
            color: [0.0, 1.0, 0.0],
        },
        Vertex {
            pos: [0.5, 0.5, 0.5],
            color: [0.0, 0.0, 1.0],
        },
        Vertex {
            pos: [-0.5, 0.5, 0.5],
            color: [1.0, 1.0, 0.0],
        },
        // Back
        Vertex {
            pos: [-0.5, -0.5, -0.5],
            color: [1.0, 0.0, 1.0],
        },
        Vertex {
            pos: [0.5, -0.5, -0.5],
            color: [0.0, 1.0, 1.0],
        },
        Vertex {
            pos: [0.5, 0.5, -0.5],
            color: [1.0, 1.0, 1.0],
        },
        Vertex {
            pos: [-0.5, 0.5, -0.5],
            color: [0.2, 0.2, 0.2],
        },
    ];

    let indices = vec![
        0, 1, 2, 2, 3, 0, 1, 5, 6, 6, 2, 1, 5, 4, 7, 7, 6, 5, 4, 0, 3, 3, 7, 4, 4, 5, 1, 1, 0, 4,
        3, 2, 6, 6, 7, 3,
    ];

    MeshData { vertices, indices }
}
