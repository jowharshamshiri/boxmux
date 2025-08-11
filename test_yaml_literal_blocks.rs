#[cfg(test)]
mod yaml_literal_block_tests {
    use serde::{Deserialize, Serialize};
    use serde_yaml;

    #[derive(Debug, Deserialize, Serialize)]
    struct TestPanel {
        pub script: Option<Vec<String>>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct TestApp {
        pub panels: Vec<TestPanel>,
    }

    #[test]
    fn test_yaml_literal_blocks() {
    println!("Testing YAML literal block deserialization in Vec<String> context...\n");

    // Test case 1: Simple string array (should work)
    let yaml1 = r#"
panels:
  - script:
      - "echo 'hello'"
      - "echo 'world'"
"#;

    println!("Test 1 - Simple string array:");
    println!("{}", yaml1);
    match serde_yaml::from_str::<TestApp>(yaml1) {
        Ok(app) => {
            println!("✅ SUCCESS: {:?}\n", app.panels[0].script);
        }
        Err(e) => {
            println!("❌ FAILED: {}\n", e);
        }
    }

    // Test case 2: Mixed simple strings and literal blocks (problematic case)
    let yaml2 = r#"
panels:
  - script:
      - "echo 'simple command'"
      - |
        if command -v free; then
          echo "RAM check"
        fi
"#;

    println!("Test 2 - Mixed simple strings and literal blocks:");
    println!("{}", yaml2);
    match serde_yaml::from_str::<TestApp>(yaml2) {
        Ok(app) => {
            println!("✅ SUCCESS: {:?}\n", app.panels[0].script);
        }
        Err(e) => {
            println!("❌ FAILED: {}\n", e);
        }
    }

    // Test case 3: Only literal blocks
    let yaml3 = r#"
panels:
  - script:
      - |
        echo "First block"
        echo "Multiple lines"
      - |
        if true; then
          echo "Second block"
        fi
"#;

    println!("Test 3 - Only literal blocks:");
    println!("{}", yaml3);
    match serde_yaml::from_str::<TestApp>(yaml3) {
        Ok(app) => {
            println!("✅ SUCCESS: {:?}\n", app.panels[0].script);
        }
        Err(e) => {
            println!("❌ FAILED: {}\n", e);
        }
    }

    // Test case 4: Investigating the raw YAML structure with serde_yaml::Value
    println!("Test 4 - Raw YAML structure analysis:");
    match serde_yaml::from_str::<serde_yaml::Value>(yaml2) {
        Ok(value) => {
            println!("Raw YAML value: {:#?}\n", value);
        }
        Err(e) => {
            println!("❌ Raw YAML parsing failed: {}\n", e);
        }
    }
    }
}