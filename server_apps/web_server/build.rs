fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tonic_prost_build::configure().compile_protos(&["proto/user_auth.proto"], &["proto"])?;
    tonic_prost_build::configure().compile_protos(&["proto/user_auth.proto"], &["proto"])?;

    Ok(())
}
