use crate::gpu::{GpuInfo, backend::GpuBackend};

pub struct MockBackend;

impl GpuBackend for MockBackend {
    fn read(&self) -> Option<GpuInfo> {
        Some(GpuInfo { temp: 0, load: 0 })
    }
    fn get_brand(&self) -> String {
        "Mock GPU".to_string()
    }
}
