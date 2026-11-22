use crate::Monitor;
use anyhow::{Context, Result};
use chrono::{Datelike, Timelike};
use hidapi::{HidApi, HidDevice};

use sysinfo::{
    Components, CpuRefreshKind, Disks, MemoryRefreshKind, Networks, RefreshKind, System,
};

pub trait DisplayTransport {
    fn send_packet(&self, packet: &[u8]) -> Result<()>;
}

pub struct HidTransport {
    pub dev: HidDevice,
}

pub struct MetricsReport {
    pub cpu_temp: u8,
    pub cpu_load: u8,
    pub cpu_speed_mhz: u16,
    pub gpu_temp: u8,
    pub gpu_load: u16,
    pub gpu_ram_used: u64,
    pub ram_load: u8,
    pub ram_used_gb: u8,
    pub storage_load: u8,
    pub storage_used_gb: u8,
    pub net_up_kb: u16,
    pub net_down_kb: u16,
}

pub struct SystemReport {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub brightness: u8,
}

pub fn discover_hid_device(vid: u16, pid: u16) -> Result<HidDevice, anyhow::Error> {
    let api = HidApi::new()?;
    let dev = api
        .device_list()
        .find(|d| d.vendor_id() == vid && d.product_id() == pid)
        .context("llTech display not found")?
        .open_device(&api)?;
    Ok(dev)
}

pub const PACKET_SIZE: usize = 64;

pub trait PacketBuilder {
    fn build_status(&self, s: &SystemReport) -> [u8; PACKET_SIZE];
    fn build_metrics(&self, m: &MetricsReport) -> [u8; PACKET_SIZE];
    fn build_flush(&self, m: &MetricsReport) -> [u8; PACKET_SIZE]; // Добавили аргумент
}

pub struct AmdPacketBuilder;

impl PacketBuilder for AmdPacketBuilder {
    // Пакет 0x03: Дата, Время и Яркость ---
    fn build_status(&self, s: &SystemReport) -> [u8; PACKET_SIZE] {
        let mut p = [0u8; PACKET_SIZE];
        p[0] = 0x03;
        p[1] = 0x01;

        // По дампу: 0x15 0x1a -> 21 и 26 (21 век, 26 год)
        p[2] = (s.year / 100 + 1) as u8; // 2026 / 100 + 1 = 21 (0x15)
        p[3] = (s.year % 100) as u8; // 2026 % 100 = 26 (0x1A)

        p[4] = s.month;
        p[5] = s.day;
        p[6] = s.hour;
        p[7] = s.minute;
        p[8] = s.second;

        p[9] = 0x06; // Статический флаг (без него мигает дисплей)
        p[10] = s.brightness; // Яркость 0-100
        p
    }
    // --- Пакет 0x02: Метрики CPU/GPU ---
    fn build_metrics(&self, m: &MetricsReport) -> [u8; PACKET_SIZE] {
        let mut p: [u8; 64] = [
            0x02, 0x14, 0x01, 0x00, 0x30, 0x02, 0x03, 0x1e, 0x03, 0x00, 0x0e, 0x04, 0x00, 0x00,
            0x05, 0x00, 0x2e, 0x06, 0x01, 0x2b, 0x07, 0x00, 0x00, 0x08, 0x00, 0x00, 0x09, 0x00,
            0x00, 0x0a, 0x0f, 0x10, 0x0b, 0x30, 0x76, 0x0c, 0x00, 0x17, 0x0d, 0x00, 0x4f, 0x0e,
            0x00, 0xed, 0x0f, 0x00, 0x30, 0x10, 0x00, 0xbd, 0x11, 0x00, 0x14, 0x12, 0x00, 0x00,
            0x13, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0x00,
        ];

        // CPU (Работает)
        p[4] = m.cpu_temp;
        p[6] = (m.cpu_speed_mhz >> 8) as u8;
        p[7] = (m.cpu_speed_mhz & 0xFF) as u8;
        p[10] = m.cpu_load;

        // GPU
        p[16] = m.gpu_temp;
        p[21] = (m.gpu_load >> 8) as u8;
        p[22] = (m.gpu_load & 0xFF) as u8;

        let gpu_ram_used = m.gpu_ram_used / 1024 / 1024;
        p[27] = (gpu_ram_used >> 8) as u8;
        p[28] = (gpu_ram_used & 0xFF) as u8;
        p[29] = 0x0a;

        // RAM Used (Индексы 30-31 из дампа 0x0f 0x10)
        let ram_mb = (m.ram_used_gb as u16) * 1024;
        p[30] = (ram_mb >> 8) as u8;
        p[31] = (ram_mb & 0xFF) as u8;
        p[32] = 0x0B;

        // RAM Load
        p[37] = m.ram_load;
        p[38] = 0x0D;

        p
    }

    // --- Пакет 0x00: Финализатор (Flush) ---
    fn build_flush(&self, m: &MetricsReport) -> [u8; PACKET_SIZE] {
        let mut p = [0u8; 64];
        let mut data: [u8; 48] = [
            0x00, 0x0d, 0x02, 0x0f, 0x12, 0x05, 0x00, 0x17, 0x07, 0x00, 0x17, 0x09, 0x00, 0x17,
            0x0a, 0x00, 0x31, 0x0b, 0x00, 0x2f, 0x0c, 0x00, 0x04, 0x0d, 0x03, 0x1d, 0x0f, 0x00,
            0x17, 0x13, 0x00, 0x00, 0x14, 0x00, 0x00, 0x17, 0x00, 0x14, 0x19, 0x00, 0x30, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        // --- СИНХРОНИЗАЦИЯ С ДИСПЛЕЕМ ---
        // 1. RAM Used (индексы 3, 4 в дампе)
        let ram_mb = (m.ram_used_gb as u16) * 1024;
        data[3] = (ram_mb >> 8) as u8;
        data[4] = (ram_mb & 0xFF) as u8;

        // 2. RAM Load (в дампе 0x17 на индексах 7, 10, 13, 28)
        // В китайских прошивках это шкала + текст
        data[7] = m.ram_load;
        data[10] = m.ram_load;
        data[13] = m.ram_load;
        data[28] = m.ram_load;

        // Внутри build_flush подставь VRAM в индексы 8-9 (для шкалы/числа)
        let gpu_vram_mb = (m.gpu_ram_used / 1024 / 1024) as u16;
        data[8] = (gpu_vram_mb >> 8) as u8;
        data[9] = (gpu_vram_mb & 0xFF) as u8;

        // 3. GPU Load (в дампе 0x04 на индексе 22)
        data[22] = (m.gpu_load & 0xFF) as u8;

        // 4. GPU Temp (в дампе 0x2f (47) на индексе 19)
        data[19] = m.gpu_temp;

        p[..48].copy_from_slice(&data);
        p
    }
}

impl DisplayTransport for HidTransport {
    fn send_packet(&self, packet: &[u8]) -> Result<()> {
        // Создаем буфер 65 байт.
        // Первый байт (0x00) — это Report ID.
        // В Windows он обязателен, в Linux hidapi использует его для выравнивания.
        let mut buf = vec![0x00];
        buf.extend_from_slice(packet);

        // Отправляем 65 байт. Устройство получит чистые 64 байта данных.
        self.dev.write(&buf).context("HID write failed")?;
        Ok(())
    }
}

pub struct MetricCollector {
    monitor: Monitor,
    sys: System,
    networks: Networks,
    disks: Disks,
    components: Components,
}

impl MetricCollector {
    pub fn new(monitor: Monitor) -> Self {
        Self {
            monitor,
            sys: System::new_with_specifics(
                RefreshKind::default()
                    .with_cpu(CpuRefreshKind::everything())
                    .with_memory(MemoryRefreshKind::everything()),
            ),
            networks: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),
            components: Components::new_with_refreshed_list(),
        }
    }

    pub fn collect(&mut self) -> MetricsReport {
        self.sys.refresh_all();

        self.networks.refresh(true);
        self.disks.refresh(true);
        self.components.refresh(true);

        self.monitor.refresh().ok();

        let mut cpu_temp = self
            .monitor
            .get_cpu_temp()
            .map(|t| t.round() as u8)
            .unwrap_or(0);

        if cpu_temp == 0 {
            cpu_temp = self
                .components
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

        let cpu_load = self.sys.global_cpu_usage() as u8;
        let cpu_speed_mhz = self
            .sys
            .cpus()
            .first()
            .map(|c| c.frequency() as u16)
            .unwrap_or(0);

        let used_mem = self.sys.used_memory();
        let total_mem = self.sys.total_memory();

        let ram_used_gb = (used_mem / 1024 / 1024 / 1024) as u8;
        let ram_load = (used_mem * 100 / total_mem.max(1)) as u8;

        let (gpu_temp, gpu_load, gpu_ram_used) = crate::gpu::read_gpu_metric();

        let main_disk = self.disks.first();
        let storage_used_gb = main_disk
            .map(|d| (d.total_space() - d.available_space()) / 1024 / 1024 / 1024)
            .unwrap_or(0) as u8;
        let storage_load = main_disk
            .map(|d| (d.total_space() - d.available_space()) * 100 / d.total_space().max(1))
            .unwrap_or(0) as u8;

        let net_down_kb =
            (self.networks.iter().map(|(_, d)| d.received()).sum::<u64>() / 1024) as u16;
        let net_up_kb = (self
            .networks
            .iter()
            .map(|(_, d)| d.transmitted())
            .sum::<u64>()
            / 1024) as u16;

        MetricsReport {
            cpu_temp,
            cpu_load,
            cpu_speed_mhz,
            gpu_temp,
            gpu_load: gpu_load as u16,
            gpu_ram_used,
            ram_load,
            ram_used_gb,
            storage_load,
            storage_used_gb,
            net_up_kb,
            net_down_kb,
        }
    }
}

pub struct SmartMonitor<'a> {
    metric_collector: &'a mut MetricCollector,
    packet_builder: &'a dyn PacketBuilder,
    display_transport: Box<dyn DisplayTransport>,
}

impl<'a> SmartMonitor<'a> {
    pub fn new(
        metric_collector: &'a mut MetricCollector,
        packet_builder: &'a dyn PacketBuilder,
        display_transport: Box<dyn DisplayTransport>,
    ) -> Self {
        Self {
            metric_collector,
            packet_builder,
            display_transport,
        }
    }

    pub fn tick(&mut self) -> Result<()> {
        let metrics = self.metric_collector.collect();
        let data_pkt = self.packet_builder.build_metrics(&metrics);
        self.display_transport.send_packet(&data_pkt)?;
        std::thread::sleep(std::time::Duration::from_millis(20)); // Быстрая пауза

        // Статус 0x03 (Яркость/Дата)
        let now = chrono::Local::now();
        let system_report = SystemReport {
            year: now.year() as u16,
            month: now.month() as u8,
            day: now.day() as u8,
            hour: now.hour() as u8,
            minute: now.minute() as u8,
            second: now.second() as u8,
            brightness: 100,
        };
        let status_pkt = self.packet_builder.build_status(&system_report);
        self.display_transport.send_packet(&status_pkt)?;

        // Flush 0x00 (ОЧИЩЕННЫЙ от старых цифр)
        std::thread::sleep(std::time::Duration::from_millis(10));
        let p_flush = self.packet_builder.build_flush(&metrics);
        self.display_transport.send_packet(&p_flush)?;

        Ok(())
    }
}
