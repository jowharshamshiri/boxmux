use std::fs::File;
use std::io::Read;
use serde_yaml;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open("layouts/dashboard_improved.yaml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // First, let's just try to parse as raw YAML value to see the structure
    let yaml_val: serde_yaml::Value = serde_yaml::from_str(&contents)?;
    println!("Successfully parsed YAML structure");

    // Let's check the app section specifically
    if let serde_yaml::Value::Mapping(root_map) = &yaml_val {
        if let Some(app_value) = root_map.get(&serde_yaml::Value::String("app".to_string())) {
            println!("Found 'app' section");
            
            if let serde_yaml::Value::Mapping(app_map) = app_value {
                // Let's check each key-value pair in the app section
                for (key, value) in app_map.iter() {
                    if let serde_yaml::Value::String(field_name) = key {
                        let value_type = match value {
                            serde_yaml::Value::Null => "null",
                            serde_yaml::Value::Bool(_) => "boolean",
                            serde_yaml::Value::Number(_) => "number",
                            serde_yaml::Value::String(_) => "string",
                            serde_yaml::Value::Sequence(_) => "sequence",
                            serde_yaml::Value::Mapping(_) => "map",
                            _ => "unknown",
                        };
                        
                        println!("Field '{}': {} ({})", field_name, value_type, 
                            if value_type == "string" { 
                                format!("'{}'", value.as_str().unwrap_or("")) 
                            } else if value_type == "map" {
                                if let serde_yaml::Value::Mapping(map) = value {
                                    format!("map with {} keys", map.len())
                                } else {
                                    "map".to_string()
                                }
                            } else { 
                                "".to_string() 
                            });
                    }
                }
            }
        }
    }

    Ok(())
}