use crate::resources::buffer::Vertex;
use crate::utils::file_io::read_file;
use anyhow::{Context, Result};
use ash::vk;

pub struct Pipeline {
    pub layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
}

fn create_shader_module(device: &ash::Device, bytes: &[u8]) -> Result<vk::ShaderModule> {
    let mut cursor = std::io::Cursor::new(bytes);
    let words = ash::util::read_spv(&mut cursor)?;

    let info = vk::ShaderModuleCreateInfo::default().code(&words);
    Ok(unsafe { device.create_shader_module(&info, None)? })
}

pub fn create_pipeline(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
    descriptor_set_layout: vk::DescriptorSetLayout,
    vert_spv: &[u8],
    frag_spv: &[u8],
) -> Result<Pipeline> {
    let vert_mod = create_shader_module(device, vert_spv).context("vert shader module")?;
    let frag_mod = create_shader_module(device, frag_spv).context("frag shader module")?;

    let main = std::ffi::CString::new("main")?;
    let stages = [
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_mod)
            .name(&main),
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_mod)
            .name(&main),
    ];

    // No vertex buffers (gl_VertexIndex)

    let binding = Vertex::binding_description();
    let attrs = Vertex::attribute_descriptions();

    let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(std::slice::from_ref(&binding))
        .vertex_attribute_descriptions(&attrs);
    let input_asm = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

    // Dynamic viewport/scissor
    let viewport_state = vk::PipelineViewportStateCreateInfo::default()
        .viewport_count(1)
        .scissor_count(1);

    let dyn_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dyn_states);

    let raster = vk::PipelineRasterizationStateCreateInfo::default()
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::NONE)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .line_width(1.0);

    let multisample = vk::PipelineMultisampleStateCreateInfo::default()
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);

    let depth = vk::PipelineDepthStencilStateCreateInfo::default()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS)
        .depth_bounds_test_enable(false)
        .stencil_test_enable(false);

    let color_blend_att = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::RGBA);

    let color_blend = vk::PipelineColorBlendStateCreateInfo::default()
        .attachments(std::slice::from_ref(&color_blend_att));

    let push_range = vk::PushConstantRange {
        stage_flags: vk::ShaderStageFlags::VERTEX,
        offset: 0,
        size: std::mem::size_of::<[[f32; 4]; 4]>() as u32,
    };

    let set_layouts = [descriptor_set_layout];
    let push_ranges = [push_range];

    let layout_info = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(&set_layouts)
        .push_constant_ranges(&push_ranges);

    let layout = unsafe { device.create_pipeline_layout(&layout_info, None)? };

    let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
        .stages(&stages)
        .vertex_input_state(&vertex_input)
        .input_assembly_state(&input_asm)
        .viewport_state(&viewport_state)
        .dynamic_state(&dynamic_state)
        .rasterization_state(&raster)
        .multisample_state(&multisample)
        .depth_stencil_state(&depth) // 👈 THIS LINE
        .color_blend_state(&color_blend)
        .layout(layout)
        .render_pass(render_pass)
        .subpass(0);

    let pipelines = unsafe {
        device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
    }
    .map_err(|(_, e)| e)?;
    let pipeline = pipelines[0];

    unsafe {
        device.destroy_shader_module(vert_mod, None);
        device.destroy_shader_module(frag_mod, None);
    }

    let _ = extent; // kept for future (swapchain recreate)
    Ok(Pipeline { layout, pipeline })
}
