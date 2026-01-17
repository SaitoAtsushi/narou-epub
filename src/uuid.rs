use std::fmt::Display;
use windows_sys::Win32::Foundation::STATUS_SUCCESS;
use windows_sys::Win32::Security::Cryptography::{
    BCRYPT_SHA1_ALGORITHM, BCryptCloseAlgorithmProvider, BCryptHash, BCryptOpenAlgorithmProvider,
};

struct Provider(*mut std::ffi::c_void);

impl Provider {
    fn new() -> Option<Self> {
        unsafe {
            let mut h_alg = std::ptr::null_mut();
            if BCryptOpenAlgorithmProvider(&mut h_alg, BCRYPT_SHA1_ALGORITHM, std::ptr::null(), 0)
                != STATUS_SUCCESS
            {
                None
            } else {
                Some(Self(h_alg))
            }
        }
    }

    fn hash(&self, name: &[u8]) -> Option<[u8; 20]> {
        const URL_NAMESPACE: [u8; 16] = 0x6ba7b811_9dad_11d1_80b4_00c04fd430c8_u128.to_be_bytes();
        let mut input_data = Vec::new();
        input_data.extend_from_slice(&URL_NAMESPACE);
        input_data.extend_from_slice(name);
        let mut hash_result = [0u8; 20];

        if unsafe {
            BCryptHash(
                self.0,
                std::ptr::null(),
                0,
                input_data.as_mut_ptr(),
                input_data.len() as u32,
                hash_result.as_mut_ptr(),
                hash_result.len() as u32,
            )
        } == STATUS_SUCCESS
        {
            Some(hash_result)
        } else {
            None
        }
    }
}

impl Drop for Provider {
    fn drop(&mut self) {
        unsafe {
            BCryptCloseAlgorithmProvider(self.0, 0);
        }
    }
}

#[derive(Clone)]
pub struct UUIDv5 {
    uuid: [u8; 16],
}

impl UUIDv5 {
    pub fn new(name: &[u8]) -> Option<Self> {
        let provider = Provider::new()?;
        let hash_result = provider.hash(name)?;
        let mut uuid_bytes = [0u8; 16];
        uuid_bytes.copy_from_slice(&hash_result[..16]);
        uuid_bytes[6] = (uuid_bytes[6] & 0x0f) | 0x50;
        uuid_bytes[8] = (uuid_bytes[8] & 0x3f) | 0x80;
        Some(Self { uuid: uuid_bytes })
    }

    #[allow(dead_code)]
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.uuid
    }
}

impl Display for UUIDv5 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.uuid[0],
            self.uuid[1],
            self.uuid[2],
            self.uuid[3],
            self.uuid[4],
            self.uuid[5],
            self.uuid[6],
            self.uuid[7],
            self.uuid[8],
            self.uuid[9],
            self.uuid[10],
            self.uuid[11],
            self.uuid[12],
            self.uuid[13],
            self.uuid[14],
            self.uuid[15]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let uuid = UUIDv5::new("python.org".as_bytes()).unwrap();
        let expected_bytes = 0x7af94e2b_4dd9_50f0_9c9a_8a48519bdef0_u128.to_be_bytes();
        let expected_string = "7af94e2b-4dd9-50f0-9c9a-8a48519bdef0";
        assert_eq!(uuid.as_bytes(), &expected_bytes);
        assert_eq!(uuid.to_string(), expected_string);

        let uuid = UUIDv5::new("https://syosetu.com/".as_bytes()).unwrap();
        let expected_bytes = 0x422d7240_e8bc_5905_b5f2_85560fd30e51_u128.to_be_bytes();
        let expected_string = "422d7240-e8bc-5905-b5f2-85560fd30e51";
        assert_eq!(uuid.as_bytes(), &expected_bytes);
        assert_eq!(uuid.to_string(), expected_string);
    }
}
