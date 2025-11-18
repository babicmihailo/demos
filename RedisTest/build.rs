// build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    prost_build::compile_protos(&["src/proto/user.proto"], &["src/proto/"])?;
    Ok(())
}
