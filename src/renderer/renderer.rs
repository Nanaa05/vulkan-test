// ===================== src/renderer/renderer.rs =====================
use super::{
    command_buffers::*,
    framebuffers::create_framebuffers,
    pipeline::{Pipeline, create_pipeline},
    render_pass::create_render_pass,
};
use crate::assets::mesh::MeshData;
use crate::core::{device::Device, surface::Surface, swapchain::Swapchain, sync::SyncObjects};
use crate::gfx::context::VkContext;
use crate::renderer::error::RenderError;
use crate::renderer::mesh::Mesh;
use crate::renderer::render_types::{FrameGlobals, RenderItem};
use crate::resources::buffer::{
    GpuBuffer, UniformBufferObject, create_index_buffer_u32, create_uniform_buffer,
    create_vertex_buffer,
};

use crate::assets::shaders;
use crate::resources::descriptor::{
    allocate_descriptor_sets, create_descriptor_pool, create_descriptor_set_layout,
    update_descriptor_sets,
};
use anyhow::Result;
use ash::vk;
use glam::Mat4;
use std::time::Instant;

pub struct Renderer {
    pub render_pass: vk::RenderPass,
    pub pipeline: Pipeline,
    pub framebuffers: Vec<vk::Framebuffer>,
    pub commands: Commands,
    pub sync: SyncObjects,
    pub current_frame: usize,
    pub frames_in_flight: usize,

    pub uniform_buffers: Vec<GpuBuffer>,
    pub uniform_mapped: Vec<*mut u8>,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,

    // depth
    pub depth_format: vk::Format,
    pub depth_image: vk::Image,
    pub depth_memory: vk::DeviceMemory,
    pub depth_view: vk::ImageView,

    // camera / control
    pub extent: vk::Extent2D,

    // optional (kept)
    pub start_time: Instant,
}

impl Renderer {
    pub fn new(dev: &Device, swap: &Swapchain, frames_in_flight: usize) -> Result<Self> {
        // --- depth resources (used by render pass + framebuffers) ---
        let (depth_format, depth_image, depth_memory, depth_view) =
            create_depth_resources(dev, swap.extent)?;

        log::info!("Using depth format: {:?}", depth_format);

        // NOTE: your render_pass.rs must be updated to accept depth_format
        let render_pass = create_render_pass(&dev.device, swap.format, depth_format)?;

        // descriptors
        let descriptor_set_layout = create_descriptor_set_layout(&dev.device)?;

        // NOTE: your pipeline.rs must enable depth testing (depthStencil state)
        let pipeline = create_pipeline(
            &dev.device,
            render_pass,
            swap.extent,
            descriptor_set_layout,
            shaders::triangle_vert_spv(),
            shaders::triangle_frag_spv(),
        )?;

        // NOTE: your framebuffers.rs must attach BOTH color and depth:
        // attachments = [color_view, depth_view]
        let framebuffers = create_framebuffers(
            &dev.device,
            render_pass,
            swap.extent,
            &swap.image_views,
            depth_view,
        )?;

        let mut uniform_buffers = Vec::with_capacity(swap.image_views.len());
        let mut uniform_mapped = Vec::with_capacity(swap.image_views.len());

        for _ in 0..swap.image_views.len() {
            let buf = create_uniform_buffer(&dev.device, &dev.memory_properties)?;

            let ptr = unsafe {
                dev.device.map_memory(
                    buf.memory,
                    0,
                    std::mem::size_of::<UniformBufferObject>() as u64,
                    vk::MemoryMapFlags::empty(),
                )?
            } as *mut u8;

            uniform_buffers.push(buf);
            uniform_mapped.push(ptr);
        }

        // descriptor pool + sets
        let descriptor_pool = create_descriptor_pool(&dev.device, swap.image_views.len() as u32)?;
        let descriptor_sets = allocate_descriptor_sets(
            &dev.device,
            descriptor_pool,
            descriptor_set_layout,
            swap.image_views.len(),
        )?;

        let uniform_vk_buffers: Vec<vk::Buffer> =
            uniform_buffers.iter().map(|b| b.buffer).collect();
        update_descriptor_sets(
            &dev.device,
            &descriptor_sets,
            &uniform_vk_buffers,
            std::mem::size_of::<UniformBufferObject>() as u64,
        );

        // commands
        let pool = create_command_pool(&dev.device, dev.queues.graphics_family)?;
        let buffers = allocate_command_buffers(&dev.device, pool, framebuffers.len() as u32)?;

        let sync = SyncObjects::new(&dev.device, swap.image_views.len(), frames_in_flight)?;

        Ok(Self {
            render_pass,
            pipeline,
            framebuffers,
            commands: Commands { pool, buffers },
            sync,
            current_frame: 0,
            frames_in_flight,

            uniform_buffers,
            uniform_mapped,
            descriptor_set_layout,
            descriptor_pool,
            descriptor_sets,

            depth_format,
            depth_image,
            depth_memory,
            depth_view,

            extent: swap.extent,

            start_time: Instant::now(),
        })
    }

    pub fn upload_mesh(&self, dev: &Device, mesh: &MeshData) -> anyhow::Result<Mesh> {
        let vertex_buffer = create_vertex_buffer(dev, &mesh.vertices)?;
        let index_buffer = create_index_buffer_u32(dev, &mesh.indices)?;

        Ok(Mesh {
            vertex_buffer,
            index_buffer,
            index_count: mesh.indices.len() as u32,
        })
    }

    pub fn draw_frame(
        &mut self,
        dev: &Device,
        _surface: &Surface,
        swap: &Swapchain,
        globals: FrameGlobals,
        items: &[RenderItem],
    ) -> Result<(), RenderError> {
        let frame = self.current_frame;

        unsafe {
            dev.device
                .wait_for_fences(&[self.sync.in_flight[frame]], true, u64::MAX)?;
        }

        let (image_index, _) = match unsafe {
            swap.loader.acquire_next_image(
                swap.swapchain,
                u64::MAX,
                self.sync.image_available[frame],
                vk::Fence::null(),
            )
        } {
            Ok(v) => v,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                return Err(RenderError::SwapchainOutOfDate);
            }
            Err(e) => return Err(RenderError::Vulkan(e)),
        };

        let idx = image_index as usize;

        // if swapchain image already in flight, wait
        if self.sync.images_in_flight[idx] != vk::Fence::null() {
            unsafe {
                dev.device
                    .wait_for_fences(&[self.sync.images_in_flight[idx]], true, u64::MAX)?;
            }
        }

        // update UBO for THIS swapchain image
        self.update_uniform(&dev.device, idx, globals.view_proj)
            .map_err(RenderError::Other)?;

        // mark image as in flight
        self.sync.images_in_flight[idx] = self.sync.in_flight[frame];

        unsafe {
            dev.device.reset_fences(&[self.sync.in_flight[frame]])?;
        }

        let cmd = self.commands.buffers[idx];

        unsafe {
            dev.device
                .reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty())?;
        }

        // --- RECORD COMMAND BUFFER (this is the missing piece) ---

        record_scene_cmd(
            &dev.device,
            cmd,
            self.render_pass,
            self.framebuffers[idx],
            swap.extent,
            self.pipeline.pipeline,
            self.pipeline.layout,
            self.descriptor_sets[idx], // ✅ still per swapchain image
            items,
        )
        .map_err(RenderError::Other)?;

        let wait_sems = [self.sync.image_available[frame]];
        let signal_sems = [self.sync.render_finished[idx]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let cmd_bufs = [cmd];

        let submit = vk::SubmitInfo::default()
            .wait_semaphores(&wait_sems)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&cmd_bufs)
            .signal_semaphores(&signal_sems);

        unsafe {
            dev.device
                .queue_submit(dev.graphics_queue, &[submit], self.sync.in_flight[frame])?;
        }

        let swapchains = [swap.swapchain];
        let image_indices = [image_index];

        let present = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal_sems)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        match unsafe { swap.loader.queue_present(dev.present_queue, &present) } {
            Ok(_) => {}
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR) => {
                return Err(RenderError::SwapchainOutOfDate);
            }
            Err(e) => return Err(RenderError::Vulkan(e)),
        }

        self.current_frame = (self.current_frame + 1) % self.sync.in_flight.len();

        Ok(())
    }

    pub fn destroy(&mut self, dev: &ash::Device) {
        unsafe {
            for (i, b) in self.uniform_buffers.iter().enumerate() {
                if !self.uniform_mapped.is_empty() {
                    dev.unmap_memory(b.memory);
                }
            }
            self.uniform_mapped.clear();

            // uniform buffers
            for b in &self.uniform_buffers {
                b.destroy(dev);
            }
            self.uniform_buffers.clear();

            // descriptors
            dev.destroy_descriptor_pool(self.descriptor_pool, None);
            dev.destroy_descriptor_set_layout(self.descriptor_set_layout, None);

            // framebuffers
            for &fb in &self.framebuffers {
                dev.destroy_framebuffer(fb, None);
            }
            self.framebuffers.clear();

            // depth
            dev.destroy_image_view(self.depth_view, None);
            dev.destroy_image(self.depth_image, None);
            dev.free_memory(self.depth_memory, None);

            // pipeline + renderpass
            dev.destroy_pipeline(self.pipeline.pipeline, None);
            dev.destroy_pipeline_layout(self.pipeline.layout, None);
            dev.destroy_render_pass(self.render_pass, None);

            // command pool
            dev.destroy_command_pool(self.commands.pool, None);
        }

        self.sync.destroy(dev);
    }

    fn update_uniform(&self, _device: &ash::Device, idx: usize, view_proj: Mat4) -> Result<()> {
        let ubo = UniformBufferObject {
            view_proj: view_proj.to_cols_array_2d(),
        };

        unsafe {
            std::ptr::copy_nonoverlapping(
                bytemuck::bytes_of(&ubo).as_ptr(),
                self.uniform_mapped[idx],
                std::mem::size_of::<UniformBufferObject>(),
            );
        }

        Ok(())
    }
}

// ----------------------- depth helpers -----------------------

fn find_memory_type_fallback(
    mem_props: &vk::PhysicalDeviceMemoryProperties,
    type_bits: u32,
    candidates: &[vk::MemoryPropertyFlags],
) -> Result<u32> {
    for &flags in candidates {
        for i in 0..mem_props.memory_type_count {
            if (type_bits & (1 << i)) != 0
                && mem_props.memory_types[i as usize]
                    .property_flags
                    .contains(flags)
            {
                return Ok(i);
            }
        }
    }
    anyhow::bail!("No suitable memory type found for depth buffer");
}

fn create_depth_resources(
    dev: &Device,
    extent: vk::Extent2D,
) -> Result<(vk::Format, vk::Image, vk::DeviceMemory, vk::ImageView)> {
    let format = dev.pick_depth_format()?; // ✅ FIXED

    let image_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .format(format)
        .extent(vk::Extent3D {
            width: extent.width,
            height: extent.height,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .initial_layout(vk::ImageLayout::UNDEFINED);

    let image = unsafe { dev.device.create_image(&image_info, None)? };
    let reqs = unsafe { dev.device.get_image_memory_requirements(image) };
    
    #[cfg(target_os = "macos")]
    let candidates = &[
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    ][..];

    #[cfg(not(target_os = "macos"))]
    let candidates = &[
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    ][..];

    let mem_index =
        find_memory_type_fallback(&dev.memory_properties, reqs.memory_type_bits, candidates)?;

    let alloc = vk::MemoryAllocateInfo::default()
        .allocation_size(reqs.size)
        .memory_type_index(mem_index);

    let memory = unsafe { dev.device.allocate_memory(&alloc, None)? };
    unsafe { dev.device.bind_image_memory(image, memory, 0)? };

    let view_info = vk::ImageViewCreateInfo::default()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .subresource_range(
            vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::DEPTH)
                .level_count(1)
                .layer_count(1),
        );

    let view = unsafe { dev.device.create_image_view(&view_info, None)? };

    Ok((format, image, memory, view))
}

impl Renderer {
    pub fn rebuild_for_swapchain(
        &mut self,
        context: &VkContext,
        swapchain: &Swapchain,
    ) -> anyhow::Result<()> {
        unsafe {
            context.device.device.device_wait_idle()?;
        }

        let frames_in_flight = self.frames_in_flight;

        self.destroy(&context.device.device);
        *self = Renderer::new(&context.device, swapchain, frames_in_flight)?;

        Ok(())
    }
}
