use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct HardwareNode {
    #[serde(rename = "Text")]
    text: String,
    #[serde(rename = "HardwareId")]
    hardware_id: Option<String>,
    #[serde(rename = "Type")]
    sensor_type: Option<String>,
    #[serde(rename = "Value")]
    value: Option<String>,
    #[serde(rename = "Children")]
    #[serde(default)]
    children: Vec<HardwareNode>,
}

pub fn get_cpu_temp(hardware_data: &HardwareNode) -> Option<f64> {
    let cpu_node = find_hardware_node("cpu", &hardware_data)?;
    let priority_sensors_names = vec![
        "CPU Package",
        "Core Max",
        "Core (Tctl/Tdie)",
        "Core Average",
    ];
    for sensor_name in priority_sensors_names.iter() {
        if let Some(sensor_raw_value) = find_sensor_value(cpu_node, sensor_name, "Temperature") {
            return parse_str_to_f64(sensor_raw_value).ok();
        }
    }

    None
}

fn parse_str_to_f64(raw_value: &str) -> Result<f64, String> {
    let split_value = raw_value.split(' ').next().ok_or("Empty string")?;
    let normalized_value = split_value.replace(',', ".");
    normalized_value
        .parse::<f64>()
        .map_err(|e| format!("Parse error '{}': {}", normalized_value, e))
}

fn find_hardware_node<'a>(
    hardware_name: &str,
    hardware_node: &'a HardwareNode,
) -> Option<&'a HardwareNode> {
    if let Some(hardware_id) = &hardware_node.hardware_id {
        if hardware_id.to_lowercase().contains(hardware_name) {
            return Some(hardware_node);
        }
    }

    for children in &hardware_node.children {
        if let Some(found) = find_hardware_node(hardware_name, children) {
            return Some(found);
        }
    }
    None
}

fn find_sensor_value<'a>(
    hardware_node: &'a HardwareNode,
    sensor_name: &str,
    sensor_type: &str,
) -> Option<&'a String> {
    if let (Some(sens_type), Some(value)) = (&hardware_node.sensor_type, &hardware_node.value) {
        if sens_type == sensor_type && sensor_name == hardware_node.text {
            return Some(value);
        }
    }

    for children in &hardware_node.children {
        if let Some(sensor_value) = find_sensor_value(children, sensor_name, sensor_type) {
            return Some(sensor_value);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_sensor_value_cpu_temperature_returns_raw_value() {
        let leaf = HardwareNode {
            text: "Core(Tctl/Tdie)".to_string(),
            value: Some("55.9 C".to_string()),
            sensor_type: Some("Temperature".to_string()),
            hardware_id: None,
            children: vec![],
        };

        let root = HardwareNode {
            sensor_type: None,
            value: None,
            text: "Temperatures".to_string(),
            hardware_id: None,
            children: vec![leaf],
        };

        let result = find_sensor_value(&root, "Core(Tctl/Tdie)", "Temperature");

        assert!(result.is_some());
        assert_eq!(result.unwrap(), "55.9 C");
    }

    #[test]
    fn parse_str_to_f64_valid_formats_success() {
        let test_cases = vec![
            ("55,9 °C", 55.9),
            ("55,9  °C", 55.9),
            ("55,9  ", 55.9),
            ("55,9", 55.9),
            ("55.9", 55.9),
        ];

        for (input, expected) in test_cases.iter() {
            let result = parse_str_to_f64(input);

            assert!(result.is_ok());
            assert!((result.unwrap() - expected).abs() < 0.001);
        }
    }

    #[test]
    fn parse_temperature_invalid_formats_returns_err() {
        let input_values = vec!["NA", " ", "", "55,9a", "55.9a"];

        for input_value in input_values.iter() {
            let result = parse_str_to_f64(input_value);

            assert!(result.is_err());
        }
    }
    #[test]
    fn find_hardware_node_case_insensitive_returns_node() {
        let hardware_ids = vec!["/amdcpu/0", "/Amdcpu/0", "AMDCPU", "amdcpu"];

        for hw_id in hardware_ids.iter() {
            let data = HardwareNode {
                text: "".to_string(),
                value: None,
                sensor_type: None,
                hardware_id: Some(hw_id.to_string()),
                children: vec![],
            };

            let result = find_hardware_node("cpu", &data);

            assert!(result.is_some());
            assert_eq!(result.unwrap().hardware_id, data.hardware_id);
        }
    }

    #[test]
    fn get_cpu_temp_check_priority_sensors_names() {
        let cpu_node = HardwareNode {
            text: "Intel Core i5-8350U".to_string(),
            hardware_id: Some("/intelcpu/0".to_string()),
            sensor_type: None,
            value: None,
            children: vec![
                HardwareNode {
                    text: "Core Max".to_string(),
                    value: Some("40,0 °C".to_string()),
                    sensor_type: Some("Temperature".to_string()),
                    hardware_id: None,
                    children: vec![],
                },
                HardwareNode {
                    text: "Core Average".to_string(),
                    value: Some("55,0 °C".to_string()),
                    sensor_type: Some("Temperature".to_string()),
                    hardware_id: None,
                    children: vec![],
                },
                HardwareNode {
                    text: "CPU Package".to_string(),
                    value: Some("56,0 °C".to_string()),
                    sensor_type: Some("Temperature".to_string()),
                    hardware_id: None,
                    children: vec![],
                },
            ],
        };

        let temp = get_cpu_temp(&cpu_node).unwrap();

        assert_eq!(temp, 56.0);
    }
}
