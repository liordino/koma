fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../proto/koma/v1/koma.proto")?;
    prost_build::Config::new().compile_protos(&["../proto/mihon/backup.proto"], &["../proto/mihon/"])?;
    Ok(())
}
