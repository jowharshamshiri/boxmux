use std::path::Path;
use boxmux::model::app::load_app_from_yaml;

fn main() {
    // Create a simple test app
    let yaml_content = r#"
app:
  layouts:
    - id: 'main'
      root: true
      title: 'Test Layout'
      children:
        - id: 'test_box'
          title: 'My Test Box'
          position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
          content: 'Hello World'
          choices:
            - label: 'Option 1'
              action: 'echo option1'
            - label: 'Option 2'
              action: 'echo option2'
"#;

    // Create temp file
    let temp_path = "/tmp/test_streamlined_tabs.yaml";
    std::fs::write(temp_path, yaml_content).unwrap();

    // Load app
    match load_app_from_yaml(Path::new(temp_path)) {
        Ok(app) => {
            let layout = app.layouts.first().unwrap();
            let muxbox = &layout.children.as_ref().unwrap()[0];
            
            println!("MuxBox ID: {}", muxbox.id);
            println!("MuxBox Title: {:?}", muxbox.title);
            println!("Tab System Streams: {}", muxbox.tab_system.streams.len());
            println!("Tab Labels: {:?}", muxbox.get_tab_labels());
            println!("Active Tab Index: {}", muxbox.get_active_tab_index());
            println!("Needs Tab Init: {}", muxbox.needs_tab_initialization());
            
            if muxbox.tab_system.streams.len() > 0 {
                for (i, stream) in muxbox.tab_system.streams.iter().enumerate() {
                    println!("  Stream {}: {} (type: {:?}, active: {})", 
                        i, stream.label, stream.source_type, stream.active);
                }
            }
        }
        Err(e) => println!("Error loading app: {}", e)
    }

    // Cleanup
    let _ = std::fs::remove_file(temp_path);
}