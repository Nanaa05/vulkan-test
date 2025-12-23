use crate::resources::buffer::GpuBuffer;

pub struct Mesh {
    pub vertex_buffer: GpuBuffer,
    pub index_buffer: GpuBuffer,
    pub index_count: u32,
}

impl Mesh {
    pub fn destroy(&self, device: &ash::Device) {
        self.vertex_buffer.destroy(device);
        self.index_buffer.destroy(device);
    }
}
