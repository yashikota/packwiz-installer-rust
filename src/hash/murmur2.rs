// 32-bit MurmurHash2 compatible with Murmur2Lib.hash32 (seed 0)
pub fn murmur2_hash(data: &[u8]) -> u32 {
    const M_32: u32 = 0x5bd1e995;
    const R_32: u32 = 24;

    let len = data.len() as u32;
    let mut h: u32 = len;

    let mut i = 0usize;
    while i + 4 <= data.len() {
        let k = (data[i] as u32)
            | ((data[i + 1] as u32) << 8)
            | ((data[i + 2] as u32) << 16)
            | ((data[i + 3] as u32) << 24);
        let mut k = k.wrapping_mul(M_32);
        k ^= k >> R_32;
        k = k.wrapping_mul(M_32);
        h = h.wrapping_mul(M_32);
        h ^= k;
        i += 4;
    }

    let left = data.len() - i;
    if left != 0 {
        if left >= 3 {
            h ^= (data[data.len() - (left - 2)] as u32) << 16;
        }
        if left >= 2 {
            h ^= (data[data.len() - (left - 1)] as u32) << 8;
        }
        if left >= 1 {
            h ^= data[data.len() - left] as u32;
        }
        h = h.wrapping_mul(M_32);
    }

    h ^= h >> 13;
    h = h.wrapping_mul(M_32);
    h ^= h >> 15;
    h
}
