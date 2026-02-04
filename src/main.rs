use smart_screen::LlTechDisplay;
use smart_screen::libhwmon::HttpDatasource;
use std::time::Duration;
use sysinfo::{Networks, System};

fn main() -> anyhow::Result<()> {
    let http_datasource = HttpDatasource::new("http://localhost:8085/data.json");
    let mut display = LlTechDisplay::auto_detect(Box::new(http_datasource))?;

    let mut sys = System::new_all();
    let mut networks = Networks::new_with_refreshed_list();

    println!("Smart Display is running...");
    display.init(&mut sys);
    loop {
        if let Err(e) = display.update_metrics(&mut sys, &mut networks) {
            eprintln!("Update error: {}", e);
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}
