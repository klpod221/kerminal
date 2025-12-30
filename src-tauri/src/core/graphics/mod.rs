pub mod kitty;
pub mod sixel;

/// Graphics protocol types and detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsProtocol {
    Sixel,
    Kitty,
    None,
}

/// Represents parsed graphics data
#[derive(Debug, Clone)]
pub struct GraphicsData {
    pub protocol: GraphicsProtocol,
    pub data: Vec<u8>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl GraphicsProtocol {
    /// Detect protocol from escape sequence
    pub fn detect(data: &str) -> Self {
        // Kitty graphics protocol: ESC _G...ESC \
        if data.contains("\x1b_G") {
            return GraphicsProtocol::Kitty;
        }

        // Sixel graphics protocol: ESC P q...ESC \
        if data.contains("\x1bP") && data.contains("q") {
            return GraphicsProtocol::Sixel;
        }

        GraphicsProtocol::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_kitty() {
        let data = "\x1b_Gf=100,a=T;base64data\x1b\\";
        assert_eq!(GraphicsProtocol::detect(data), GraphicsProtocol::Kitty);
    }

    #[test]
    fn test_detect_sixel() {
        let data = "\x1bPq\"1;1;100;100#0;2;0;0;0#1;2;100;100;0\x1b\\";
        assert_eq!(GraphicsProtocol::detect(data), GraphicsProtocol::Sixel);
    }

    #[test]
    fn test_detect_none() {
        let data = "regular terminal text";
        assert_eq!(GraphicsProtocol::detect(data), GraphicsProtocol::None);
    }
}
