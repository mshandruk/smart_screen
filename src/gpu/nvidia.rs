use crate::gpu::{GpuInfo, backend::GpuBackend};
use nvml_wrapper::Nvml;

pub struct NvidiaBackend {
    nvml: Nvml,
}

impl NvidiaBackend {
    pub fn new() -> Option<Self> {
        Some(Self {
            nvml: Nvml::init().ok()?,
        })
    }
}

impl GpuBackend for NvidiaBackend {
    fn read(&self) -> Option<GpuInfo> {
        let device = self.nvml.device_by_index(0).ok()?;

        let temp = device
            .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
            .ok()?
            .min(255) as u8;

        let load = device.utilization_rates().ok()?.gpu.min(255) as u8;

        let mem_info = device.memory_info().ok()?;
        let ram_used = mem_info.used;

        Some(GpuInfo {
            temp,
            load,
            ram_used,
        })
    }

    fn get_brand(&self) -> String {
        self.nvml
            .device_by_index(0)
            .and_then(|dev| dev.name())
            .unwrap_or_else(|_| "NVIDIA GPU".to_string())
    }
}
