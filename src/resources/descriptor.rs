use anyhow::Result;
use ash::vk;

pub fn create_descriptor_set_layout(device: &ash::Device) -> Result<vk::DescriptorSetLayout> {
    let binding = vk::DescriptorSetLayoutBinding::default()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX);

    let info =
        vk::DescriptorSetLayoutCreateInfo::default().bindings(std::slice::from_ref(&binding));

    Ok(unsafe { device.create_descriptor_set_layout(&info, None)? })
}

pub fn create_descriptor_pool(device: &ash::Device, count: u32) -> Result<vk::DescriptorPool> {
    let pool_size = vk::DescriptorPoolSize::default()
        .ty(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(count);

    let info = vk::DescriptorPoolCreateInfo::default()
        .pool_sizes(std::slice::from_ref(&pool_size))
        .max_sets(count);

    Ok(unsafe { device.create_descriptor_pool(&info, None)? })
}

pub fn allocate_descriptor_sets(
    device: &ash::Device,
    pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,
    count: usize,
) -> Result<Vec<vk::DescriptorSet>> {
    let layouts = vec![layout; count];
    let info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(&layouts);

    Ok(unsafe { device.allocate_descriptor_sets(&info)? })
}

pub fn update_descriptor_sets(
    device: &ash::Device,
    sets: &[vk::DescriptorSet],
    uniform_buffers: &[vk::Buffer],
    range: vk::DeviceSize,
) {
    for (i, &set) in sets.iter().enumerate() {
        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(uniform_buffers[i])
            .offset(0)
            .range(range);

        let write = vk::WriteDescriptorSet::default()
            .dst_set(set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(std::slice::from_ref(&buffer_info));

        unsafe {
            device.update_descriptor_sets(std::slice::from_ref(&write), &[]);
        }
    }
}
