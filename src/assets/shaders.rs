pub fn triangle_vert_spv() -> &'static [u8] {
    include_bytes!(concat!(env!("OUT_DIR"), "/triangle.vert.spv"))
}

pub fn triangle_frag_spv() -> &'static [u8] {
    include_bytes!(concat!(env!("OUT_DIR"), "/triangle.frag.spv"))
}
