use anyhow::Result;
use ash::Entry;

pub fn create_entry() -> Result<Entry> {
    // Default ash behavior dynamically loads Vulkan (Entry::load). :contentReference[oaicite:2]{index=2}
    let entry = unsafe { Entry::load()? };
    Ok(entry)
}

