use crate::Monitor;
use crate::libhwmon::DataSource;
use anyhow::{Context, Result};
use hidapi::{HidApi, HidDevice};
use sysinfo::{Components, Networks, System};

pub const VID: u16 = 0x0483;
pub const PID: u16 = 0x0065;
pub const PACKET_SIZE: usize = 64;

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum MetricId {
    CpuTemp = 0x03,
    CpuLoad = 0x04,
    GpuTemp = 0x06,
    GpuLoad = 0x08,
    RamLoad = 0x0A,
    CpuModel = 0x0B,
    NetUp = 0x12,
    NetDown = 0x13,
    Volume = 0x14,
}

pub struct LlTechDisplay {
    dev: HidDevice,
    monitor: Monitor,
}

impl LlTechDisplay {
    pub fn auto_detect(datasource: Box<dyn DataSource>) -> Result<Self> {
        let monitor = Monitor::new(datasource);
        let api = HidApi::new()?;
        let dev = api
            .device_list()
            .find(|d| d.vendor_id() == VID && d.product_id() == PID)
            .context("llTech display not found")?
            .open_device(&api)?;

        Ok(Self { dev, monitor })
    }

    pub fn init(&self, sys: &mut System) {
        sys.refresh_cpu_all();
        if let Some(cpu) = sys.cpus().first() {
            let brand = cpu.brand();
            println!("CPU Brand: {}", brand);
        }

        let gpu_brand = crate::gpu::read_gpu_brand();
        println!("GPU Brand: {}", gpu_brand);

        let components = Components::new_with_refreshed_list();
        println!("=> components:");
        for component in &components {
            println!("{component:?}");
        }
    }

    pub fn update_metrics(&mut self, sys: &mut System, networks: &mut Networks) -> Result<()> {
        sys.refresh_all();
        networks.refresh(false);

        // 1. Temperature CPU (Priority: LHM JSON -> sysinfo)
        self.monitor.refresh().ok();
        let mut cpu_temp = self
            .monitor
            .get_cpu_temp()
            .map(|t| t.round() as u8)
            .unwrap_or(0);

        if cpu_temp == 0 {
            println!("cpu_temp from sysinfo");
            let components = Components::new_with_refreshed_list();
            cpu_temp = components
                .iter()
                .find(|c| {
                    let l = c.label().to_uppercase();
                    l.contains("CPU")
                        || l.contains("CORE")
                        || l.contains("PACKAGE")
                        || l.contains("TCTL")
                })
                .and_then(|c| c.temperature())
                .unwrap_or(0.0) as u8;
        }

        // 2. Networks (Mbps)
        let net_down_raw: u64 = networks.iter().map(|(_, d)| d.received()).sum();
        let net_up_raw: u64 = networks.iter().map(|(_, d)| d.transmitted()).sum();
        let net_down = ((net_down_raw as f64 * 8.0) / 1_000_000.0).min(255.0) as u8;
        let net_up = ((net_up_raw as f64 * 8.0) / 1_000_000.0).min(255.0) as u8;

        // 3. System
        let cpu_load = sys.global_cpu_usage() as u8;
        let ram_load = (sys.used_memory() * 100 / sys.total_memory().max(1)) as u8;
        let (gpu_temp, gpu_load) = crate::gpu::read_gpu_metric();
        let volume = 50u8;

        self.send_metrics_fixed(
            cpu_temp, cpu_load, gpu_temp, gpu_load, ram_load, net_up, net_down, volume,
        )
    }

    fn send_metrics_fixed(
        &self,
        cpu_temp: u8,
        cpu_load: u8,
        gpu_temp: u8,
        gpu_load: u8,
        ram_load: u8,
        net_up: u8,
        net_down: u8,
        volume: u8,
    ) -> Result<()> {
        let metrics_data: [u8; PACKET_SIZE] = [
            0x02,
            0x14,
            0x01,
            0x00,
            cpu_temp,
            MetricId::CpuTemp as u8,
            0x0e,
            0x07,
            0x03,
            0x00,
            cpu_load,
            MetricId::CpuLoad as u8,
            0x00,
            0x00,
            0x05,
            0x00,
            gpu_temp,
            MetricId::GpuTemp as u8,
            0x01,
            0x2b,
            0x07,
            0x00,
            gpu_load,
            MetricId::GpuLoad as u8,
            0x00,
            0x00,
            0x09,
            0x00,
            ram_load,
            MetricId::RamLoad as u8,
            0x18,
            0x96,
            0x0b,
            0x26,
            0xf0,
            0x0c,
            0x00,
            0x26,
            0x0d,
            0x00,
            0x50,
            0x0e,
            0x00,
            0xed,
            0x0f,
            0x00,
            0x31,
            0x10,
            0x00,
            0xbc,
            0x11,
            0x00,
            0x14,
            0x12,
            0x00,
            net_up,
            0x13,
            0x00,
            net_down,
            0x14,
            0x00,
            volume,
            0x00,
            0x00,
        ];

        if cfg!(target_os = "windows") {
            let mut win_buf = [0u8; PACKET_SIZE + 1];
            win_buf[1..].copy_from_slice(&metrics_data);
            self.dev.write(&win_buf)?;
        } else {
            self.dev.write(&metrics_data)?;
        }

        println!(
            "Send: CPU:{}°C/{}% GPU:{}°C/{}% Net:↑{}/↓{} Mbps",
            cpu_temp, cpu_load, gpu_temp, gpu_load, net_up, net_down
        );

        Ok(())
    }
}
