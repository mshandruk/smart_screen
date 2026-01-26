use libhwmon::{HardwareNode, get_cpu_temp};

#[test]
fn get_cpu_temp_amd_returns_value() {
    let json_data = r#"{
  "Text": "Sensor",
  "Value": "Value",
  "Children": [
    {
      "Text": "DESKTOP-2AOQT87",
      "Value": "",
      "Children": [
        {
          "Text": "AMD Ryzen 7 5700X",
          "Value": "",
          "HardwareId": "/amdcpu/0",
          "Children": [
            {
              "Text": "Temperatures",
              "Value": "",
              "Children": [
                {
                  "Text": "Core (Tctl/Tdie)",
                  "Value": "55,9 \u00B0C",
                  "SensorId": "/amdcpu/0/temperature/2",
                  "Type": "Temperature",
                  "Children": []
                },
                {
                  "Text": "CCD1 (Tdie)",
                  "Value": "45,5 \u00B0C",
                  "SensorId": "/amdcpu/0/temperature/3",
                  "Type": "Temperature",
                  "Children": []
                }
              ]
            }
          ]
        }
      ]
    }
  ]
}"#;
    let deserialized: HardwareNode = serde_json::from_str(json_data).unwrap();

    let cpu_temp = get_cpu_temp(&deserialized);

    assert!(cpu_temp.is_some());
    assert!((cpu_temp.unwrap() - 55.9).abs() < 0.001);
}

#[test]
fn get_cpu_temp_intel_returns_value() {
    let json_data = r#"{
  "id": 0,
  "Text": "Sensor",
  "Min": "Min",
  "Value": "Value",
  "Max": "Max",
  "ImageURL": "",
  "Children": [
    {
      "id": 1,
      "Text": "DESKTOP-UHUOKO0",
      "Min": "",
      "Value": "",
      "Max": "",
      "ImageURL": "images_icon/computer.png",
      "Children": [
        {
          "id": 3,
          "Text": "Intel Core i5-8350U",
          "Min": "",
          "Value": "",
          "Max": "",
          "HardwareId": "/intelcpu/0",
          "ImageURL": "images_icon/cpu.png",
          "Children": [
            {
              "id": 21,
              "Text": "Temperatures",
              "Min": "",
              "Value": "",
              "Max": "",
              "ImageURL": "images_icon/temperature.png",
              "Children": [
                {
                  "id": 22,
                  "Text": "Core Max",
                  "Min": "40,0 °C",
                  "Value": "45,0 °C",
                  "Max": "65,0 °C",
                  "SensorId": "/intelcpu/0/temperature/0",
                  "Type": "Temperature",
                  "RawMin": "40,0 °C",
                  "RawValue": "45,0 °C",
                  "RawMax": "65,0 °C",
                  "ImageURL": "images/transparent.png",
                  "Children": []
                },
                {
                  "id": 23,
                  "Text": "Core Average",
                  "Min": "40,0 °C",
                  "Value": "44,3 °C",
                  "Max": "63,8 °C",
                  "SensorId": "/intelcpu/0/temperature/1",
                  "Type": "Temperature",
                  "RawMin": "40,0 °C",
                  "RawValue": "44,3 °C",
                  "RawMax": "63,8 °C",
                  "ImageURL": "images/transparent.png",
                  "Children": []
                },
                {
                  "id": 28,
                  "Text": "CPU Package",
                  "Min": "41,0 °C",
                  "Value": "45,0 °C",
                  "Max": "64,0 °C",
                  "SensorId": "/intelcpu/0/temperature/6",
                  "Type": "Temperature",
                  "RawMin": "41,0 °C",
                  "RawValue": "45,0 °C",
                  "RawMax": "64,0 °C",
                  "ImageURL": "images/transparent.png",
                  "Children": []
                }
              ]
            }
          ]
        }
      ]
    }
  ]
}"#;

    let deserialized: HardwareNode = serde_json::from_str(json_data).unwrap();

    let cpu_temp = get_cpu_temp(&deserialized);

    assert!(cpu_temp.is_some());
    assert!((cpu_temp.unwrap() - 45.0).abs() < 0.001);
}

#[test]
fn get_cpu_temp_from_empty_json_returns_none() {
    let json_data = r#"{"Text": ""}"#;
    let deserialized: HardwareNode = serde_json::from_str(json_data).unwrap();
    let cpu_temp = get_cpu_temp(&deserialized);
    assert!(cpu_temp.is_none());
}

#[test]
fn get_cpu_temp_broken_value_returns_none() {
    let json_data = r#"{
        "Text": "AMD CPU",
        "HardwareId": "/amdcpu/0",
        "Children": [{
            "Text": "Temperatures",
            "Children": [
                { "Text": "Core (Tctl/Tdie)", "Value": "ERROR", "Type": "Temperature" }
            ]
        }]
    }"#;
    let node: HardwareNode = serde_json::from_str(json_data).unwrap();
    let result = get_cpu_temp(&node);
    assert!(result.is_none());
}
