use smart_screen::LlTechDisplay;
use std::time::Duration;
use sysinfo::{Networks, System};

fn main() -> anyhow::Result<()> {
    let display = LlTechDisplay::auto_detect()?;

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
