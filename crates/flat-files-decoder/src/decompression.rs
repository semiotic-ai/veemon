#[derive(Clone, Copy, Debug, Default)]
pub enum Decompression {
    Zstd,
    #[default]
    None,
}

impl From<&str> for Decompression {
    fn from(value: &str) -> Self {
        match value {
            "true" | "1" => Decompression::Zstd,
            "false" | "0" => Decompression::None,
            _ => Decompression::None,
        }
    }
}

impl From<bool> for Decompression {
    fn from(value: bool) -> Self {
        match value {
            true => Decompression::Zstd,
            false => Decompression::None,
        }
    }
}
