use smart_monitor::discover_hid_device;
use smart_monitor::libhwmon::{HttpDatasource, Monitor};
use smart_monitor::lltech::{AmdPacketBuilder, HidTransport, MetricCollector, SmartMonitor};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

pub const VID: u16 = 0x0483;
pub const PID: u16 = 0x0065;

// Статический флаг для управления состоянием сервиса
static RUNNING: AtomicBool = AtomicBool::new(true);

fn main_logic() -> anyhow::Result<()> {
    let http_datasource = HttpDatasource::new("http://localhost:8085/data.json");
    let lib_hw_monitor = Monitor::new(Box::new(http_datasource));
    let mut metric_collector = MetricCollector::new(lib_hw_monitor);
    let packet_builder = AmdPacketBuilder {};

    while RUNNING.load(Ordering::SeqCst) {
        let dev = match discover_hid_device(VID, PID) {
            Ok(dev) => dev,
            Err(e) => {
                println!("Failed to discover hid device: {}", e);
                thread::sleep(Duration::from_secs(5));
                continue;
            }
        };

        let mut smart_monitor = SmartMonitor::new(
            &mut metric_collector,
            &packet_builder,
            Box::new(HidTransport { dev }),
        );

        println!("Smart-monitor started...");
        while RUNNING.load(Ordering::SeqCst) && smart_monitor.tick().is_ok() {
            thread::sleep(Duration::from_secs(5));
        }

        if RUNNING.load(Ordering::SeqCst) {
            println!("Connection lost, searching again...");
        }
    }
    println!("Graceful shutdown complete.");
    Ok(())
}

#[cfg(windows)]
fn main() -> anyhow::Result<()> {
    use windows_service::{
        define_windows_service,
        service::{
            ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType,
        },
        service_control_handler::{self, ServiceControlHandlerResult},
        service_dispatcher,
    };

    define_windows_service!(ffi_service_main, smart_monitor_service_main);

    fn smart_monitor_service_main(_arguments: Vec<std::ffi::OsString>) {
        let event_handler = move |control_event| -> ServiceControlHandlerResult {
            match control_event {
                windows_service::service::ServiceControl::Stop => {
                    RUNNING.store(false, Ordering::SeqCst);
                    ServiceControlHandlerResult::NoError
                }
                _ => ServiceControlHandlerResult::NotImplemented,
            }
        };

        if let Ok(status_handle) = service_control_handler::register("SmartMonitor", event_handler)
        {
            let _ = status_handle.set_service_status(ServiceStatus {
                service_type: ServiceType::OWN_PROCESS,
                current_state: ServiceState::Running,
                controls_accepted: ServiceControlAccept::STOP,
                exit_code: ServiceExitCode::Win32(0),
                checkpoint: 0,
                wait_hint: Duration::default(),
                process_id: None,
            });

            let _ = main_logic();

            let _ = status_handle.set_service_status(ServiceStatus {
                service_type: ServiceType::OWN_PROCESS,
                current_state: ServiceState::Stopped,
                controls_accepted: ServiceControlAccept::empty(),
                exit_code: ServiceExitCode::Win32(0),
                checkpoint: 0,
                wait_hint: Duration::default(),
                process_id: None,
            });
        }
    }

    if service_dispatcher::start("SmartMonitor", ffi_service_main).is_err() {
        main_logic()?;
    }
    Ok(())
}

#[cfg(not(windows))]
fn main() -> anyhow::Result<()> {
    main_logic()
}
