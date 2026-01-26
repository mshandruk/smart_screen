use super::GpuInfo;

pub trait GpuBackend: Send + Sync {
    fn read(&self) -> Option<GpuInfo>;
    fn get_brand(&self) -> String;
}
