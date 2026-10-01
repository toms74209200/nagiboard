pub fn generate() -> Result<uuid::Uuid, String> {
    let mut bytes = [0u8; 16];
    std::io::Read::read_exact(
        &mut std::fs::File::open("/dev/urandom").map_err(|e| e.to_string())?,
        &mut bytes,
    )
    .map_err(|e| e.to_string())?;
    Ok(uuid::Builder::from_random_bytes(bytes).into_uuid())
}
