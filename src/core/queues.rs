#[derive(Clone, Copy, Debug)]
pub struct QueueFamilyIndices {
    pub graphics_family: u32,
    pub present_family: u32,
}

impl QueueFamilyIndices {
    pub fn same_family(&self) -> bool {
        self.graphics_family == self.present_family
    }
}
