//! An example boxmux plugin offering one component, `gauge`: a horizontal bar per configured
//! value, sized to the box.
//!
//! Configuration:
//!
//! ```yaml
//! plugin_component: gauge
//! plugin_config:
//!   gauges:
//!     - {label: CPU, value: 42, max: 100}
//! ```

use serde::Deserialize;
use std::ffi::{c_char, CStr, CString};

#[derive(Deserialize)]
struct Request {
    component: String,
    config: Config,
    width: usize,
    height: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    gauges: Vec<Gauge>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Gauge {
    label: String,
    value: f64,
    #[serde(default = "default_max")]
    max: f64,
}

fn default_max() -> f64 {
    100.0
}

fn render(request: &Request) -> Result<String, String> {
    if request.component != "gauge" {
        return Err(format!("unknown component '{}'", request.component));
    }
    let label_width = request
        .config
        .gauges
        .iter()
        .map(|g| g.label.chars().count())
        .max()
        .unwrap_or(0);
    let mut lines = Vec::new();
    for gauge in request.config.gauges.iter().take(request.height) {
        if gauge.max <= 0.0 {
            return Err(format!("gauge '{}' has a max of {}", gauge.label, gauge.max));
        }
        let share = (gauge.value / gauge.max).clamp(0.0, 1.0);
        let percent = format!("{:>4.0}%", share * 100.0);
        // label, a space, [bar], a space, percent
        let bar_width = request
            .width
            .saturating_sub(label_width + percent.len() + 4)
            .max(1);
        let filled = (share * bar_width as f64).round() as usize;
        lines.push(format!(
            "{:<label_width$} [{}{}] {}",
            gauge.label,
            "█".repeat(filled),
            "░".repeat(bar_width - filled),
            percent,
        ));
    }
    Ok(lines.join("\n"))
}

fn reply(result: Result<String, String>) -> *mut c_char {
    let json = match result {
        Ok(content) => serde_json::json!({ "content": content }),
        Err(error) => serde_json::json!({ "error": error }),
    };
    CString::new(json.to_string())
        .expect("JSON has no NUL bytes")
        .into_raw()
}

#[no_mangle]
pub extern "C" fn boxmux_plugin_abi_version() -> u32 {
    1
}

#[no_mangle]
pub extern "C" fn boxmux_plugin_describe() -> *mut c_char {
    CString::new(r#"{"name": "gauge", "components": ["gauge"]}"#)
        .expect("no NUL bytes")
        .into_raw()
}

/// # Safety
/// `request` is a NUL-terminated string that boxmux keeps alive during the call.
#[no_mangle]
pub unsafe extern "C" fn boxmux_plugin_render(request: *const c_char) -> *mut c_char {
    let text = match CStr::from_ptr(request).to_str() {
        Ok(text) => text,
        Err(_) => return reply(Err("the request is not UTF-8".into())),
    };
    match serde_json::from_str::<Request>(text) {
        Ok(request) => reply(render(&request)),
        Err(e) => reply(Err(format!("invalid configuration: {}", e))),
    }
}

/// # Safety
/// `text` was returned by this plugin and is freed once.
#[no_mangle]
pub unsafe extern "C" fn boxmux_plugin_free(text: *mut c_char) {
    drop(CString::from_raw(text));
}
