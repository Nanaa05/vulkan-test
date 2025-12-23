use std::{env, fs, path::PathBuf};

fn main() {
    // Rebuild if shader sources change
    println!("cargo:rerun-if-changed=shaders/triangle.vert");
    println!("cargo:rerun-if-changed=shaders/triangle.frag");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let mut compiler = shaderc::Compiler::new().expect("shaderc compiler");
    let mut options = shaderc::CompileOptions::new().expect("shaderc options");
    options.set_optimization_level(shaderc::OptimizationLevel::Performance);

    compile_one(
        &mut compiler,
        &options,
        "shaders/triangle.vert",
        shaderc::ShaderKind::Vertex,
        out_dir.join("triangle.vert.spv"),
    );

    compile_one(
        &mut compiler,
        &options,
        "shaders/triangle.frag",
        shaderc::ShaderKind::Fragment,
        out_dir.join("triangle.frag.spv"),
    );
}

fn compile_one(
    compiler: &mut shaderc::Compiler,
    options: &shaderc::CompileOptions,
    path: &str,
    kind: shaderc::ShaderKind,
    out_path: PathBuf,
) {
    let source = std::fs::read_to_string(path).expect("read shader");
    let artifact = compiler
        .compile_into_spirv(&source, kind, path, "main", Some(options))
        .expect("compile shader to SPIR-V");

    fs::write(out_path, artifact.as_binary_u8()).expect("write .spv");
}
