#[inline(always)]
pub fn read_u64(ptr: *const u8, offset: usize) -> u64 {
    let ptr = unsafe { ptr.add(offset) };
    unsafe { std::ptr::read(ptr as *const u64) }
}

#[inline(always)]
pub fn read_f64(ptr: *const u8, offset: usize) -> f64 {
    let ptr = unsafe { ptr.add(offset) };
    unsafe { std::ptr::read(ptr as *const f64) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_u64() {
        let data: &[u8] = &[1, 0, 0, 0, 0, 0, 0, 0];
        let value = read_u64(data.as_ptr(), 0);
        assert_eq!(value, 1);
    }

    #[test]
    fn test_read_f64() {
        let data: &[u8] = &[0, 0, 0, 0, 0, 0, 240, 63];
        let value = read_f64(data.as_ptr(), 0);
        assert_eq!(value, 1.0);
    }
}