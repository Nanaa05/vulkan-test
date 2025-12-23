pub fn vk_ok(r: ash::vk::Result) -> Result<(), ash::vk::Result> {
    if r == ash::vk::Result::SUCCESS {
        Ok(())
    } else {
        Err(r)
    }
}

