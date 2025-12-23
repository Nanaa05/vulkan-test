use anyhow::Result;
use ash::vk;

pub fn create_framebuffers(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    extent: vk::Extent2D,
    image_views: &[vk::ImageView],
    depth_view: vk::ImageView,
) -> Result<Vec<vk::Framebuffer>> {
    let mut fbs = Vec::with_capacity(image_views.len());

    for &view in image_views {
        let attachments = [view, depth_view];

        let info = vk::FramebufferCreateInfo::default()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(extent.width)
            .height(extent.height)
            .layers(1);

        let fb = unsafe { device.create_framebuffer(&info, None)? };
        fbs.push(fb);
    }

    Ok(fbs)
}
