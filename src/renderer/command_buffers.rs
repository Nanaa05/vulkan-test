use crate::resources::buffer::GpuBuffer;
use anyhow::Result;
use ash::vk;
pub struct Commands {
    pub pool: vk::CommandPool,
    pub buffers: Vec<vk::CommandBuffer>,
}
use crate::renderer::mesh::Mesh;

pub fn create_command_pool(device: &ash::Device, graphics_family: u32) -> Result<vk::CommandPool> {
    let info = vk::CommandPoolCreateInfo::default()
        .queue_family_index(graphics_family)
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

    Ok(unsafe { device.create_command_pool(&info, None)? })
}

pub fn allocate_command_buffers(
    device: &ash::Device,
    pool: vk::CommandPool,
    count: u32,
) -> Result<Vec<vk::CommandBuffer>> {
    let info = vk::CommandBufferAllocateInfo::default()
        .command_pool(pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(count);

    Ok(unsafe { device.allocate_command_buffers(&info)? })
}

pub fn record_triangle_cmd(
    device: &ash::Device,
    cmd: vk::CommandBuffer,
    render_pass: vk::RenderPass,
    framebuffer: vk::Framebuffer,
    extent: vk::Extent2D,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_set: vk::DescriptorSet,
    mesh: &Mesh,
    model: glam::Mat4,
) -> Result<()> {
    let begin = vk::CommandBufferBeginInfo::default();
    unsafe { device.begin_command_buffer(cmd, &begin)? };
    let model_bytes = model.to_cols_array_2d();

    let clear_values = [
        vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.05, 0.05, 0.08, 1.0],
            },
        },
        vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            },
        },
    ];

    let rp_begin = vk::RenderPassBeginInfo::default()
        .render_pass(render_pass)
        .framebuffer(framebuffer)
        .render_area(vk::Rect2D::default().extent(extent))
        .clear_values(&clear_values);

    unsafe {
        device.cmd_begin_render_pass(cmd, &rp_begin, vk::SubpassContents::INLINE);

        device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, pipeline);

        device.cmd_push_constants(
            cmd,
            pipeline_layout,
            vk::ShaderStageFlags::VERTEX,
            0,
            bytemuck::bytes_of(&model_bytes),
        );

        device.cmd_bind_descriptor_sets(
            cmd,
            vk::PipelineBindPoint::GRAPHICS,
            pipeline_layout,
            0,
            &[descriptor_set],
            &[],
        );

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        let scissor = vk::Rect2D::default().extent(extent);

        device.cmd_set_viewport(cmd, 0, &[viewport]);
        device.cmd_set_scissor(cmd, 0, &[scissor]);

        device.cmd_bind_vertex_buffers(cmd, 0, &[mesh.vertex_buffer.buffer], &[0]);
        device.cmd_bind_index_buffer(cmd, mesh.index_buffer.buffer, 0, vk::IndexType::UINT32);
        device.cmd_draw_indexed(cmd, mesh.index_count, 1, 0, 0, 0);

        device.cmd_end_render_pass(cmd);
        device.end_command_buffer(cmd)?;
    }

    Ok(())
}
