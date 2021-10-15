use thiserror::Error;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OtherError {
    details: String,
}

impl std::error::Error for OtherError {}

impl std::fmt::Display for OtherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl OtherError {
    pub fn new(msg: &str) -> OtherError {
        OtherError {
            details: msg.to_string(),
        }
    }
}

#[derive(Debug, Error)]
pub enum RendererError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Surface(#[from] wgpu::SurfaceError),
    #[error(transparent)]
    WGpu(#[from] wgpu::Error),
    #[error(transparent)]
    Device(#[from] wgpu::RequestDeviceError),
    #[error(transparent)]
    ImageError(#[from] image::ImageError),
    #[error(transparent)]
    Other(#[from] OtherError),
}
