//! Plugins: shared libraries that draw the content of boxes with a `plugin_component`.
//!
//! A plugin is a C-ABI shared library listed under the app's `plugins` setting. Only strings
//! cross the library boundary, as UTF-8 JSON in NUL-terminated buffers:
//!
//! ```c
//! uint32_t boxmux_plugin_abi_version(void);          // must return 1
//! char *boxmux_plugin_describe(void);                // {"name": "...", "components": ["..."]}
//! char *boxmux_plugin_render(const char *request);   // see below
//! void boxmux_plugin_free(char *text);               // frees strings returned above
//! ```
//!
//! A render request is `{"component": "...", "config": {...}, "width": N, "height": N}`; the
//! reply is `{"content": "lines of text"}` or `{"error": "what went wrong"}`. The content may
//! contain SGR color sequences and is shown in the box's content area.

use libloading::Library;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::{c_char, CStr, CString};
use std::path::{Path, PathBuf};

/// The plugin interface version this boxmux speaks.
pub const ABI_VERSION: u32 = 1;

type AbiVersionFn = unsafe extern "C" fn() -> u32;
type DescribeFn = unsafe extern "C" fn() -> *mut c_char;
type RenderFn = unsafe extern "C" fn(*const c_char) -> *mut c_char;
type FreeFn = unsafe extern "C" fn(*mut c_char);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Description {
    name: String,
    components: Vec<String>,
}

#[derive(Serialize)]
struct RenderRequest<'a> {
    component: &'a str,
    config: &'a serde_json::Value,
    width: usize,
    height: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RenderReply {
    content: Option<String>,
    error: Option<String>,
}

struct Plugin {
    name: String,
    path: PathBuf,
    render: RenderFn,
    free: FreeFn,
    // Declared last so it is dropped after the function pointers above stop being used.
    _library: Library,
}

impl Plugin {
    fn load(path: &Path) -> Result<(Plugin, Vec<String>), String> {
        let fail = |what: String| format!("plugin {}: {}", path.display(), what);
        // SAFETY: loading a library runs its initializers; the user listed it in the
        // configuration as a boxmux plugin, which is the trust decision.
        let library = unsafe { Library::new(path) }.map_err(|e| fail(e.to_string()))?;
        let symbol = |name: &str| -> Result<*const (), String> {
            let with_nul = format!("{}\0", name);
            // SAFETY: the address is only turned into a function pointer of the signature the
            // plugin interface defines for `name`.
            unsafe { library.get::<*const ()>(with_nul.as_bytes()) }
                .map(|address| *address)
                .map_err(|e| fail(format!("missing {}: {}", name, e)))
        };
        // SAFETY: each address is a function exported under the name the plugin interface
        // gives it, with the signature it defines.
        let (abi_version, describe, render, free) = unsafe {
            (
                std::mem::transmute::<*const (), AbiVersionFn>(symbol(
                    "boxmux_plugin_abi_version",
                )?),
                std::mem::transmute::<*const (), DescribeFn>(symbol("boxmux_plugin_describe")?),
                std::mem::transmute::<*const (), RenderFn>(symbol("boxmux_plugin_render")?),
                std::mem::transmute::<*const (), FreeFn>(symbol("boxmux_plugin_free")?),
            )
        };
        // SAFETY: the plugin interface defines this function as taking no arguments.
        let version = unsafe { abi_version() };
        if version != ABI_VERSION {
            return Err(fail(format!(
                "speaks plugin interface version {}, boxmux speaks {}",
                version, ABI_VERSION
            )));
        }
        // SAFETY: describe takes no arguments and returns a string freed with `free`.
        let text = unsafe { take_string(describe(), free) }.map_err(fail)?;
        let description: Description = serde_json::from_str(&text)
            .map_err(|e| fail(format!("invalid description {}: {}", text, e)))?;
        Ok((
            Plugin {
                name: description.name,
                path: path.to_path_buf(),
                render,
                free,
                _library: library,
            },
            description.components,
        ))
    }
}

/// Copies a string returned by a plugin and gives it back to the plugin to free.
///
/// # Safety
/// `text` must be null or a NUL-terminated string allocated by the plugin whose `free` is
/// given.
unsafe fn take_string(text: *mut c_char, free: FreeFn) -> Result<String, String> {
    if text.is_null() {
        return Err("returned no string".to_string());
    }
    let copied = CStr::from_ptr(text).to_str().map(str::to_string);
    free(text);
    copied.map_err(|_| "returned a string that is not UTF-8".to_string())
}

/// The loaded plugins and the components they draw.
#[derive(Default)]
pub struct Plugins {
    plugins: Vec<Plugin>,
    /// Component type to the index of the plugin that draws it.
    components: HashMap<String, usize>,
}

impl std::fmt::Debug for Plugins {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Plugins")
            .field(
                "plugins",
                &self.plugins.iter().map(|p| &p.name).collect::<Vec<_>>(),
            )
            .field("components", &self.components.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl Plugins {
    /// Loads every library in `paths`. Fails when a library cannot be loaded, speaks another
    /// interface version, or offers a component another plugin already offers.
    pub fn load(paths: &[PathBuf]) -> Result<Plugins, String> {
        let mut loaded = Plugins::default();
        for path in paths {
            let (plugin, components) = Plugin::load(path)?;
            let index = loaded.plugins.len();
            for component in components {
                if let Some(&other) = loaded.components.get(&component) {
                    return Err(format!(
                        "plugin {}: component '{}' is also offered by {}",
                        path.display(),
                        component,
                        loaded.plugins[other].path.display()
                    ));
                }
                loaded.components.insert(component, index);
            }
            loaded.plugins.push(plugin);
        }
        Ok(loaded)
    }

    pub fn offers(&self, component: &str) -> bool {
        self.components.contains_key(component)
    }

    /// The text the plugin draws for `component` in a `width` x `height` area.
    pub fn render(
        &self,
        component: &str,
        config: Option<&HashMap<String, serde_json::Value>>,
        width: usize,
        height: usize,
    ) -> Result<String, String> {
        let index = *self
            .components
            .get(component)
            .ok_or_else(|| "no loaded plugin offers this component".to_string())?;
        let plugin = &self.plugins[index];
        let config =
            serde_json::to_value(config.cloned().unwrap_or_default()).map_err(|e| e.to_string())?;
        let request = serde_json::to_string(&RenderRequest {
            component,
            config: &config,
            width,
            height,
        })
        .map_err(|e| e.to_string())?;
        let request = CString::new(request).map_err(|e| e.to_string())?;
        // SAFETY: render takes a NUL-terminated string, which outlives the call, and returns a
        // string freed with the plugin's free.
        let reply = unsafe { take_string((plugin.render)(request.as_ptr()), plugin.free) }?;
        let reply: RenderReply =
            serde_json::from_str(&reply).map_err(|e| format!("invalid reply {}: {}", reply, e))?;
        match (reply.content, reply.error) {
            (Some(content), None) => Ok(content),
            (None, Some(error)) => Err(error),
            _ => Err("the reply must have either content or error".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    /// A plugin compiled from Rust source, in its own directory.
    fn build_plugin(dir: &Path, name: &str, source: &str) -> PathBuf {
        let source_path = dir.join(format!("{}.rs", name));
        std::fs::write(&source_path, source).unwrap();
        let output = dir.join(libloading::library_filename(name));
        let status = Command::new("rustc")
            .args(["--crate-type", "cdylib", "--edition", "2021", "-o"])
            .arg(&output)
            .arg(&source_path)
            .status()
            .expect("rustc runs");
        assert!(status.success(), "the test plugin compiles");
        output
    }

    const CLOCK: &str = r##"
use std::ffi::{c_char, CStr, CString};

#[no_mangle]
pub extern "C" fn boxmux_plugin_abi_version() -> u32 { 1 }

#[no_mangle]
pub extern "C" fn boxmux_plugin_describe() -> *mut c_char {
    CString::new(r#"{"name": "clock", "components": ["clock", "broken"]}"#).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn boxmux_plugin_render(request: *const c_char) -> *mut c_char {
    let request = unsafe { CStr::from_ptr(request) }.to_str().unwrap();
    let reply = if request.contains("\"component\":\"broken\"") {
        r#"{"error": "sensor offline"}"#.to_string()
    } else {
        // Echo the size and config back so the test sees what was sent.
        format!(r#"{{"content": "request {}"}}"#, request.replace('"', "'"))
    };
    CString::new(reply).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn boxmux_plugin_free(text: *mut c_char) {
    unsafe { drop(CString::from_raw(text)) };
}
"##;

    #[test]
    fn a_plugin_renders_its_components_and_reports_its_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = build_plugin(dir.path(), "clock", CLOCK);
        let plugins = Plugins::load(&[path]).unwrap();
        assert!(plugins.offers("clock") && plugins.offers("broken") && !plugins.offers("x"));

        let config: HashMap<String, serde_json::Value> =
            [("zone".to_string(), serde_json::json!("UTC"))].into();
        let text = plugins.render("clock", Some(&config), 30, 4).unwrap();
        assert!(
            text.contains("'width':30") && text.contains("'height':4"),
            "{}",
            text
        );
        assert!(text.contains("'zone':'UTC'"), "{}", text);

        assert_eq!(
            plugins.render("broken", None, 10, 1),
            Err("sensor offline".to_string())
        );
        assert!(plugins.render("missing", None, 10, 1).is_err());
    }

    #[test]
    fn a_plugin_of_another_interface_version_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let source = CLOCK.replace("-> u32 { 1 }", "-> u32 { 7 }");
        let path = build_plugin(dir.path(), "future", &source);
        let error = Plugins::load(&[path]).unwrap_err();
        assert!(error.contains("version 7"), "{}", error);
    }

    #[test]
    fn two_plugins_cannot_offer_the_same_component() {
        let dir = tempfile::tempdir().unwrap();
        let first = build_plugin(dir.path(), "first", CLOCK);
        let second = build_plugin(dir.path(), "second", CLOCK);
        let error = Plugins::load(&[first, second]).unwrap_err();
        assert!(error.contains("'clock'"), "{}", error);
    }

    #[test]
    fn a_library_that_is_not_a_plugin_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = build_plugin(
            dir.path(),
            "plain",
            "#[no_mangle] pub extern \"C\" fn f() {}",
        );
        let error = Plugins::load(&[path]).unwrap_err();
        assert!(error.contains("boxmux_plugin_abi_version"), "{}", error);
    }
}
