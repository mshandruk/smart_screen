use smart_screen::libhwmon::{DataSource, DataSourceError, HardwareNode, Monitor};
use std::default::Default;

struct MockDataSource {
    result: Result<HardwareNode, DataSourceError>,
}

impl DataSource for MockDataSource {
    fn get_data(&self) -> Result<HardwareNode, DataSourceError> {
        self.result.clone()
    }
}

fn get_monitor(node: HardwareNode) -> Monitor {
    let mock_data_source = MockDataSource {
        result: Ok(node.clone()),
    };
    Monitor::new(Box::new(mock_data_source))
}

fn create_sensor(sensor_type: &str, name: &str, value: &str) -> HardwareNode {
    HardwareNode {
        text: name.to_string(),
        sensor_type: Some(sensor_type.to_string()),
        value: Some(value.to_string()),
        ..Default::default()
    }
}
fn create_hardware_node(
    hardware_name: &str,
    hardware_id: &str,
    children: Vec<HardwareNode>,
) -> HardwareNode {
    HardwareNode {
        text: hardware_name.to_string(),
        hardware_id: Some(hardware_id.to_string()),
        children,
        ..Default::default()
    }
}

fn wrap_into_node(node_name: &str, children: Vec<HardwareNode>) -> HardwareNode {
    HardwareNode {
        text: node_name.to_string(),
        children,
        ..Default::default()
    }
}

#[test]
fn get_cpu_temp_amd_returns_value() {
    let sensor = create_sensor("Temperature", "Core (Tctl/Tdie)", "55,9 °C");
    let group = wrap_into_node("Temperatures", vec![sensor]);
    let cpu_node = create_hardware_node("AMD Ryzen 7 5700X", "/amdcpu/0", vec![group]);
    let desktop = wrap_into_node("DESKTOP-2AOQT87", vec![cpu_node]);
    let root = wrap_into_node("Sensor", vec![desktop]);
    let mut monitor = get_monitor(root);
    monitor.refresh().unwrap();

    let cpu_temp = monitor.get_cpu_temp();

    assert!(cpu_temp.is_some());
    assert!((cpu_temp.unwrap() - 55.9).abs() < 0.001);
}

#[test]
fn get_cpu_temp_intel_returns_value() {
    let group = wrap_into_node(
        "Temperatures",
        vec![
            create_sensor("Temperature", "Core Max", "45,9 °C"),
            create_sensor("Temperature", "Core Average", "44,3 °C"),
            create_sensor("Temperature", "CPU Package", "45,0 °C"),
        ],
    );
    let cpu_node = create_hardware_node("AMD Ryzen 7 5700X", "/amdcpu/0", vec![group]);
    let desktop = wrap_into_node("DESKTOP-2AOQT87", vec![cpu_node]);
    let root = wrap_into_node("Sensor", vec![desktop]);
    let mut monitor = get_monitor(root);
    monitor.refresh().unwrap();

    let cpu_temp = monitor.get_cpu_temp();

    assert!(cpu_temp.is_some());
    assert!((cpu_temp.unwrap() - 45.0).abs() < 0.001);
}

#[test]
fn get_cpu_temp_with_invalid_value_data_returns_none() {
    let group = wrap_into_node(
        "Temperatures",
        vec![create_sensor(
            "Temperature",
            "Core Max",
            "Invalid temperature value",
        )],
    );
    let cpu_node = create_hardware_node("AMD Ryzen 7 5700X", "/amdcpu/0", vec![group]);
    let desktop = wrap_into_node("DESKTOP-2AOQT87", vec![cpu_node]);
    let root = wrap_into_node("Sensor", vec![desktop]);
    let mut monitor = get_monitor(root);
    monitor.refresh().unwrap();

    let cpu_temp = monitor.get_cpu_temp();

    assert!(cpu_temp.is_none());
}

#[test]
fn get_cpu_temp_with_others_data_returns_correct_value() {
    let temperatures = wrap_into_node(
        "Temperatures",
        vec![
            create_sensor("Temperature", "CPU Package", "55,0 °C"),
            create_sensor("Temperature", "Core Average", "45.0 °C"),
        ],
    );
    let load = wrap_into_node("Load", vec![create_sensor("Load", "CPU Total", "10,2 %")]);

    let cpu_node = create_hardware_node(
        "Intel Core i5-8350U",
        "/intelcpu/0",
        vec![temperatures, load],
    );

    let desktop = wrap_into_node("DESKTOP-2AOQT87", vec![cpu_node]);
    let root = wrap_into_node("Sensor", vec![desktop]);
    let mut monitor = get_monitor(root);
    monitor.refresh().unwrap();

    let cpu_temp = monitor.get_cpu_temp();

    assert!(cpu_temp.is_some());
    assert!((cpu_temp.unwrap() - 55.0).abs() < 0.001);
}

#[test]
fn get_cpu_temp_with_fake_and_real_cpu_returns_correct_value() {
    let temperatures = wrap_into_node(
        "Temperatures",
        vec![create_sensor("Temperature", "CPU Package", "55,0 °C")],
    );

    let fake_cpu = create_hardware_node("Intel Core i5-8350U", "/intelcpu/0", vec![]);
    let real_cpu = create_hardware_node("Intel Core i5-8350U", "/intelcpu/1", vec![temperatures]);
    let desktop = wrap_into_node("DESKTOP-2AOQT87", vec![fake_cpu, real_cpu]);
    let root = wrap_into_node("Sensor", vec![desktop]);
    let mut monitor = get_monitor(root);
    monitor.refresh().unwrap();

    let cpu_temp = monitor.get_cpu_temp();

    assert!(cpu_temp.is_some());
    assert!((cpu_temp.unwrap() - 55.0).abs() < 0.001);
}
