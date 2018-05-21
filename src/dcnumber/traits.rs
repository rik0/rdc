pub trait FromBytes: Sized {
    type Err;

    fn from_bytes_radix(bytes: &[u8], radix: u32) -> Result<Self, Self::Err>;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Err> {
        Self::from_bytes_radix(bytes, 10)
    }
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::Err> {
        Self::from_bytes_radix(s.as_ref(), radix)
    }
}
