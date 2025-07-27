use std::io::Read;

pub fn read_u64(bytes: &mut &[u8]) -> anyhow::Result<u64> {
    let mut buf: [u8; 8] = [0; 8];
    bytes.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

pub fn read_f64(bytes: &mut &[u8]) -> anyhow::Result<f64> {
    let mut buf: [u8; 8] = [0; 8];
    bytes.read_exact(&mut buf)?;
    Ok(f64::from_le_bytes(buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_u64() {
        let mut data: &[u8] = &[1, 0, 0, 0, 0, 0, 0, 0];
        let value = read_u64(&mut data).unwrap();
        assert_eq!(value, 1);
    }

    #[test]
    fn test_read_f64() {
        let mut data: &[u8] = &[0, 0, 0, 0, 0, 0, 240, 63];
        let value = read_f64(&mut data).unwrap();
        assert_eq!(value, 1.0);
    }
}