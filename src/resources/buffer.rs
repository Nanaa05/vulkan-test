use crate::core::device::Device;
use anyhow::Result;
use ash::vk;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(std::mem::size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    }

    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        [
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT) // 👈 vec3
                .offset(0),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(12),
        ]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UniformBufferObject {
    pub view_proj: [[f32; 4]; 4],
}

pub struct GpuBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub size: vk::DeviceSize,
}

pub fn create_vertex_buffer(dev: &Device, vertices: &[Vertex]) -> Result<GpuBuffer> {
    if vertices.is_empty() {
        anyhow::bail!("create_vertex_buffer called with 0 vertices");
    }

    let size = (std::mem::size_of::<Vertex>() * vertices.len()) as u64;
    let bytes = bytemuck::cast_slice(vertices); // &[u8]

    create_device_local_buffer_with_staging(dev, size, vk::BufferUsageFlags::VERTEX_BUFFER, bytes)
}

fn find_memory_type(
    mem_props: &vk::PhysicalDeviceMemoryProperties,
    type_filter: u32,
    props: vk::MemoryPropertyFlags,
) -> Result<u32> {
    for i in 0..mem_props.memory_type_count {
        if (type_filter & (1 << i)) != 0
            && mem_props.memory_types[i as usize]
                .property_flags
                .contains(props)
        {
            return Ok(i);
        }
    }
    anyhow::bail!("No suitable memory type found");
}

impl GpuBuffer {
    /// Destroys the Vulkan buffer and frees its memory.
    ///
    /// # Safety / Requirements
    /// The GPU must not be using this buffer anymore (e.g., device idle or proper sync).
    pub fn destroy(&self, device: &ash::Device) {
        unsafe {
            device.destroy_buffer(self.buffer, None);
            device.free_memory(self.memory, None);
        }
    }
}

pub fn create_device_local_buffer_with_staging(
    dev: &Device,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    src_bytes: &[u8],
) -> Result<GpuBuffer> {
    if src_bytes.is_empty() || size == 0 {
        anyhow::bail!("staging upload called with empty data");
    }
    if src_bytes.len() as u64 != size {
        anyhow::bail!(
            "size mismatch: size={} but src_bytes={}",
            size,
            src_bytes.len()
        );
    }

    // 1) staging buffer (CPU visible)
    let staging = create_buffer(
        &dev.device,
        &dev.memory_properties,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )?;

    unsafe {
        let data = dev
            .device
            .map_memory(staging.memory, 0, size, vk::MemoryMapFlags::empty())?;
        std::ptr::copy_nonoverlapping(src_bytes.as_ptr(), data as *mut u8, src_bytes.len());
        dev.device.unmap_memory(staging.memory);
    }

    // 2) device-local buffer (GPU only)
    let dst = create_buffer(
        &dev.device,
        &dev.memory_properties,
        size,
        usage | vk::BufferUsageFlags::TRANSFER_DST,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    // 3) copy staging -> device local
    let cmd = dev.begin_one_time_commands()?;
    unsafe {
        let region = vk::BufferCopy::default().size(size);
        dev.device
            .cmd_copy_buffer(cmd, staging.buffer, dst.buffer, &[region]);
    }
    dev.end_one_time_commands(cmd)?;

    // 4) cleanup staging
    staging.destroy(&dev.device);

    Ok(dst)
}

pub fn create_buffer(
    device: &ash::Device,
    mem_props: &vk::PhysicalDeviceMemoryProperties,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    props: vk::MemoryPropertyFlags,
) -> Result<GpuBuffer> {
    if size == 0 {
        anyhow::bail!("create_buffer called with size=0");
    }

    let buffer_info = vk::BufferCreateInfo::default()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = unsafe { device.create_buffer(&buffer_info, None)? };
    let reqs = unsafe { device.get_buffer_memory_requirements(buffer) };

    let mem_index = find_memory_type(mem_props, reqs.memory_type_bits, props)?;

    let alloc = vk::MemoryAllocateInfo::default()
        .allocation_size(reqs.size)
        .memory_type_index(mem_index);

    let memory = unsafe { device.allocate_memory(&alloc, None)? };
    unsafe { device.bind_buffer_memory(buffer, memory, 0)? };

    Ok(GpuBuffer {
        buffer,
        memory,
        size,
    })
}

pub fn create_uniform_buffer(
    device: &ash::Device,
    mem_props: &vk::PhysicalDeviceMemoryProperties,
) -> Result<GpuBuffer> {
    create_buffer(
        device,
        mem_props,
        std::mem::size_of::<UniformBufferObject>() as u64,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )
}

pub fn create_index_buffer_u32(dev: &Device, indices: &[u32]) -> Result<GpuBuffer> {
    if indices.is_empty() {
        anyhow::bail!("create_index_buffer called with 0 indices");
    }

    let size = (std::mem::size_of::<u32>() * indices.len()) as u64;
    let bytes = bytemuck::cast_slice(indices); // &[u8]

    create_device_local_buffer_with_staging(dev, size, vk::BufferUsageFlags::INDEX_BUFFER, bytes)
}
