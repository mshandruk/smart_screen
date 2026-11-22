use backend::GpuBackend;
use std::sync::LazyLock;

mod backend;
mod mock;
mod nvidia;

pub struct GpuInfo {
    pub temp: u8,
    pub load: u8,
    pub ram_used: u64,
}

static BACKEND: LazyLock<Box<dyn GpuBackend + Send + Sync>> = LazyLock::new(|| {
    if let Some(nvidia) = nvidia::NvidiaBackend::new() {
        Box::new(nvidia)
    } else {
        Box::new(mock::MockBackend)
    }
});

pub fn read_gpu_metric() -> (u8, u8, u64) {
    BACKEND
        .read()
        .map(|info| (info.temp, info.load, info.ram_used))
        .unwrap_or((0, 0, 0))
}

pub fn read_gpu_brand() -> String {
    BACKEND.get_brand()
}
