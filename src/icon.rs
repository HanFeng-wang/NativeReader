use std::fs;
use std::path::PathBuf;

const ICON_BYTES: &[u8] = include_bytes!("../assets/native-reader.ico");
const ICON_FILE: &str = "native-reader.ico";

pub fn install() -> Option<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("NativeReader");
    fs::create_dir_all(&base).ok()?;
    let path = base.join(ICON_FILE);
    if fs::read(&path).ok().as_deref() != Some(ICON_BYTES) {
        fs::write(&path, ICON_BYTES).ok()?;
    }
    Some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_icon_is_a_windows_icon() {
        assert!(ICON_BYTES.len() > 6);
        assert_eq!(u16::from_le_bytes([ICON_BYTES[0], ICON_BYTES[1]]), 0);
        assert_eq!(u16::from_le_bytes([ICON_BYTES[2], ICON_BYTES[3]]), 1);
        assert!(u16::from_le_bytes([ICON_BYTES[4], ICON_BYTES[5]]) >= 5);
    }
}
