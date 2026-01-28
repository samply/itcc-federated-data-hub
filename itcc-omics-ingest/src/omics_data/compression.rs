use crate::utils::error_type::ErrorType;

pub fn compress_zstd(input: &[u8], level: &i32) -> Result<Vec<u8>, ErrorType> {
    zstd::stream::encode_all(input, *level).map_err(|_| ErrorType::CompressFile)
}
