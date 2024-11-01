#[derive(Clone, Copy, Debug, Default)]
pub enum Compression {
    Zstd,
    #[default]
    None,
}

impl From<&str> for Compression {
    fn from(value: &str) -> Self {
        match value {
            "true" | "1" => Compression::Zstd,
            "false" | "0" => Compression::None,
            _ => Compression::None,
        }
    }
}

impl From<bool> for Compression {
    fn from(value: bool) -> Self {
        match value {
            true => Compression::Zstd,
            false => Compression::None,
        }
    }
}
