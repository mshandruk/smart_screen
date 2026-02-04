use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct HardwareNode {
    #[serde(rename = "Text")]
    pub text: String,
    #[serde(rename = "HardwareId")]
    pub hardware_id: Option<String>,
    #[serde(rename = "Type")]
    pub sensor_type: Option<String>,
    #[serde(rename = "Value")]
    pub value: Option<String>,
    #[serde(rename = "Children")]
    pub children: Vec<HardwareNode>,
}

#[derive(Debug, Clone)]
pub enum DataSourceError {
    Network(String),
    InvalidData(String),
}
pub trait DataSource {
    fn get_data(&self) -> Result<HardwareNode, DataSourceError>;
}

pub struct HttpDatasource {
    agent: ureq::Agent,
    url: String,
}

impl HttpDatasource {
    pub fn new(url: &str) -> Self {
        let config = ureq::config::Config::builder()
            .timeout_connect(Some(Duration::from_millis(150)))
            .timeout_global(Some(Duration::from_millis(300)))
            .build();
        Self {
            agent: ureq::Agent::new_with_config(config),
            url: url.to_string(),
        }
    }
}

impl DataSource for HttpDatasource {
    fn get_data(&self) -> Result<HardwareNode, DataSourceError> {
        let body = self
            .agent
            .get(&self.url)
            .call()
            .map_err(|e| DataSourceError::Network(e.to_string()))?
            .body_mut()
            .read_to_string()
            .map_err(|e| DataSourceError::InvalidData(e.to_string()))?;

        serde_json::from_str(&body).map_err(|e| DataSourceError::InvalidData(e.to_string()))
    }
}

pub struct Monitor {
    data_source: Box<dyn DataSource>,
    cache: Option<HardwareNode>,
}

impl Monitor {
    pub fn new(data_source: Box<dyn DataSource>) -> Self {
        Self {
            data_source,
            cache: None,
        }
    }
    pub fn refresh(&mut self) -> Result<(), DataSourceError> {
        let data = self.data_source.get_data()?;
        self.cache = Some(data);
        Ok(())
    }
    pub fn get_cpu_temp(&self) -> Option<f64> {
        let priority_sensors_names = [
            "CPU Package",
            "Core Max",
            "Core (Tctl/Tdie)",
            "Core Average",
        ];

        let data = self.cache.as_ref()?;
        SensorFinder::new(data)
            .find("cpu", "temperature", &priority_sensors_names)
            .and_then(|node| node.value.as_ref())
            .and_then(|raw_value| parse_str_to_f64(raw_value).ok())
    }
}

struct SensorFinder<'a> {
    root: &'a HardwareNode,
}

impl<'a> SensorFinder<'a> {
    pub fn new(root: &'a HardwareNode) -> Self {
        Self { root }
    }
    pub fn find(
        &self,
        hardware_id: &str,
        target_type: &str,
        target_names: &[&str],
    ) -> Option<&HardwareNode> {
        Self::find_recursive(self.root, hardware_id, target_type, target_names)
    }
    fn find_recursive(
        current_node: &'a HardwareNode,
        hardware_id: &str,
        target_type: &str,
        target_names: &[&str],
    ) -> Option<&'a HardwareNode> {
        println!("find_recursive посетил: {}", current_node.text);
        let is_correct_hw = current_node
            .hardware_id
            .as_ref()
            .map(|hw_id| hw_id.to_lowercase().contains(hardware_id))
            .unwrap_or(false);

        if is_correct_hw {
            for target_name in target_names {
                if let Some(found_sensor) =
                    Self::find_sensor(current_node, target_type, target_name)
                {
                    return Some(found_sensor);
                }
            }
            return None;
        }

        for child in &current_node.children {
            if let Some(found) = Self::find_recursive(child, hardware_id, target_type, target_names)
            {
                return Some(found);
            }
        }
        None
    }

    fn find_sensor(
        hardware_node: &'a HardwareNode,
        target_type: &str,
        target_name: &str,
    ) -> Option<&'a HardwareNode> {
        if let Some(sensor_type) = &hardware_node.sensor_type {
            if target_name.to_lowercase() == hardware_node.text.to_lowercase()
                && target_type.to_lowercase() == sensor_type.to_lowercase()
            {
                return Some(hardware_node);
            }
        }

        for child in &hardware_node.children {
            if let Some(found_sensor) = Self::find_sensor(child, target_type, target_name) {
                return Some(found_sensor);
            }
        }
        None
    }
}

fn parse_str_to_f64(raw_value: &str) -> Result<f64, String> {
    let split_value = raw_value.split(' ').next().ok_or("Empty string")?;
    let normalized_value = split_value.replace(',', ".");
    normalized_value
        .parse::<f64>()
        .map_err(|e| format!("Parse error '{}': {}", normalized_value, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockDataSource {
        result: Result<HardwareNode, DataSourceError>,
    }

    impl DataSource for MockDataSource {
        fn get_data(&self) -> Result<HardwareNode, DataSourceError> {
            self.result.clone()
        }
    }
    #[test]
    fn get_data_network_failure_returns_none() {
        let mock_datasource = MockDataSource {
            result: Err(DataSourceError::Network("Timeout".into())),
        };
        let monitor = Monitor::new(Box::new(mock_datasource));

        let result = monitor.get_cpu_temp();

        assert!(result.is_none());
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
}
