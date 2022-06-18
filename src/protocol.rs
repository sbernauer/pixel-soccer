use async_trait::async_trait;

#[derive(Debug)]
pub enum PixelflutRequest {
    GetSize,
    /// Layout of rgb: 8 bits padding, 8 bits r, 8 bits g, 8 bits green
    SetPixel {
        x: u16,
        y: u16,
        rgb: u32,
    },
    GetPixel {
        x: u16,
        y: u16,
    },
}

#[derive(Debug)]
pub enum PixelflutResponse {
    Size {
        width: u16,
        height: u16,
    },
    /// Layout of rgb: 8 bits padding, 8 bits r, 8 bits g, 8 bits green
    Pixel {
        x: u16,
        y: u16,
        rgb: u32,
    },
}

#[async_trait]
pub trait Serialize {
    fn serialize(&self, vec: &mut Vec<u8>);
}

impl Serialize for PixelflutRequest {
    fn serialize(&self, vec: &mut Vec<u8>) {
        match self {
            PixelflutRequest::GetSize => vec.extend_from_slice("SIZE\n".as_bytes()),
            PixelflutRequest::SetPixel { x, y, rgb } => {
                vec.extend_from_slice(format!("PX {x} {y} {rgb:06x}\n").as_bytes())
            }
            PixelflutRequest::GetPixel { x, y } => {
                vec.extend_from_slice(format!("PX {x} {y}\n").as_bytes())
            }
        }
    }
}
