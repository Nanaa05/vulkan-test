use anyhow::Result;
use ash::vk;

pub struct SyncObjects {
    pub image_available: Vec<vk::Semaphore>,
    pub in_flight: Vec<vk::Fence>,
    pub images_in_flight: Vec<vk::Fence>,
    pub render_finished: Vec<vk::Semaphore>,
}

impl SyncObjects {
    pub fn new(device: &ash::Device, image_count: usize, frames_in_flight: usize) -> Result<Self> {
        let sem_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let mut image_available = Vec::with_capacity(frames_in_flight);
        let mut in_flight = Vec::with_capacity(frames_in_flight);

        for _ in 0..frames_in_flight {
            unsafe {
                image_available.push(device.create_semaphore(&sem_info, None)?);
                in_flight.push(device.create_fence(&fence_info, None)?);
            }
        }

        let mut render_finished = Vec::with_capacity(image_count);
        for _ in 0..image_count {
            unsafe {
                render_finished.push(device.create_semaphore(&sem_info, None)?);
            }
        }

        Ok(Self {
            image_available,
            in_flight,
            images_in_flight: vec![vk::Fence::null(); image_count],
            render_finished,
        })
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            // Semaphores signaled when an image is available
            for &s in &self.image_available {
                device.destroy_semaphore(s, None);
            }

            // Semaphores signaled when rendering is finished
            for &s in &self.render_finished {
                device.destroy_semaphore(s, None);
            }

            // Fences for frames-in-flight
            for &f in &self.in_flight {
                device.destroy_fence(f, None);
            }
        }

        self.image_available.clear();
        self.render_finished.clear();
        self.in_flight.clear();
        self.images_in_flight.clear();
    }
}
