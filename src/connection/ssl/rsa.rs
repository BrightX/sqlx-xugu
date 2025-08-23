use num_bigint::BigUint;

pub fn rsa_encrypt(data: &[u8], public_key: &[u8], trail_key: &[u8]) -> Vec<u8> {
    let exponent = BigUint::from_bytes_le(public_key);
    let modulus = BigUint::from_bytes_le(trail_key);

    // 预分配输出缓冲区（避免多次扩容）
    let block_count = (data.len() + 31) / 32;
    let mut cipher_data = Vec::with_capacity(block_count * 32);

    // 使用chunks_exact处理完整块
    for block in data.chunks_exact(32) {
        let encrypted = rsa_encrypt_block(block, &exponent, &modulus);
        cipher_data.extend_from_slice(&encrypted);
    }

    // 处理剩余部分
    let rem = data.chunks_exact(32).remainder();
    if !rem.is_empty() {
        let mut last_block = [0u8; 32];
        last_block[..rem.len()].copy_from_slice(rem);
        let encrypted = rsa_encrypt_block(&last_block, &exponent, &modulus);
        cipher_data.extend_from_slice(&encrypted);
    }

    cipher_data
}

fn rsa_encrypt_block(block: &[u8], exponent: &BigUint, modulus: &BigUint) -> Vec<u8> {
    let plain_num = BigUint::from_bytes_le(block);
    let cipher_num = plain_num.modpow(exponent, modulus);

    let mut bytes = cipher_num.to_bytes_le();
    let mut remaining = block.len() - bytes.len();

    while remaining > 0 {
        bytes.push(0);
        remaining -= 1;
    }
    bytes
}
