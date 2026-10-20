use crate::model::bounds::{layout_bounds, Bounds};
use crate::model::common::Config;
use crate::model::layout::Layout;
use crate::model::muxbox::MuxBox;
use crate::model::variables::Scope;
use crate::validation::{validate_schema, validate_semantics};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateRoot {
    pub app: App,
}

/// A plugin library to load. A relative path is relative to the configuration file.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PluginSpec {
    pub path: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct App {
    pub layouts: Vec<Layout>,
    #[serde(default)]
    pub libs: Option<Vec<String>>,
    #[serde(default)]
    pub on_keypress: Option<HashMap<String, Vec<String>>>,
    #[serde(default)]
    pub hot_keys: Option<HashMap<String, String>>,
    #[serde(
        default,
        deserialize_with = "crate::model::variables::deserialize_scope"
    )]
    pub variables: Option<Scope>,
    /// The shell scripts run in.
    #[serde(default)]
    pub shell: crate::platform::Shell,
    /// Plugin libraries drawing `plugin_component` boxes.
    #[serde(default)]
    pub plugins: Option<Vec<PluginSpec>>,
    #[serde(skip)]
    pub adjusted_bounds: Option<HashMap<String, HashMap<String, Bounds>>>,
    /// The cells of the screen the app is drawn on. Set by the runtime from the display's size.
    #[serde(skip, default = "App::initial_screen")]
    screen: Bounds,
}

impl PartialEq for App {
    fn eq(&self, other: &Self) -> bool {
        self.layouts == other.layouts
            && self.libs == other.libs
            && self.on_keypress == other.on_keypress
            && self.hot_keys == other.hot_keys
            && self.variables == other.variables
            && self.shell == other.shell
            && self.plugins == other.plugins
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        App {
            layouts: Vec::new(),
            libs: None,
            on_keypress: None,
            hot_keys: None,
            variables: None,
            shell: crate::platform::Shell::default(),
            plugins: None,
            adjusted_bounds: None,
            screen: Self::initial_screen(),
        }
    }

    /// The screen before a display reports its size: a standard 80x24 terminal.
    fn initial_screen() -> Bounds {
        Bounds::new(0, 0, 79, 23)
    }

    /// The cells of the screen the app is drawn on.
    pub fn screen(&self) -> Bounds {
        self.screen
    }

    /// Records the size of the screen. Every computed bounds is recomputed for the new size.
    /// A screen without cells cannot show anything and is rejected.
    pub fn set_screen_size(&mut self, cols: u16, rows: u16) -> Result<(), String> {
        if cols == 0 || rows == 0 {
            return Err(format!("the screen has no cells ({}x{})", cols, rows));
        }
        let screen = Bounds::new(0, 0, usize::from(cols) - 1, usize::from(rows) - 1);
        if screen != self.screen {
            self.screen = screen;
            self.adjusted_bounds = None;
        }
        Ok(())
    }

    /// Forgets the cached box graph and computed bounds. Call after boxes are added, removed or
    /// replaced, and after the terminal is resized.
    pub fn invalidate_layout_cache(&mut self) {
        self.adjusted_bounds = None;
    }

    pub fn get_adjusted_bounds(
        &mut self,
        force_readjust: Option<bool>,
    ) -> &HashMap<String, HashMap<String, Bounds>> {
        if self.adjusted_bounds.is_none() || force_readjust.unwrap_or(false) {
            self.adjusted_bounds = Some(self.calculate_bounds());
        }
        self.adjusted_bounds
            .as_ref()
            .expect("adjusted bounds were computed above")
    }

    pub fn get_layout_by_id(&self, id: &str) -> Option<&Layout> {
        self.layouts.iter().find(|l| l.id == id)
    }

    pub fn get_layout_by_id_mut(&mut self, id: &str) -> Option<&mut Layout> {
        self.layouts.iter_mut().find(|l| l.id == id)
    }

    /// The layout shown at start: the one marked `root`, or the first.
    pub fn get_root_layout(&self) -> Option<&Layout> {
        self.layouts
            .iter()
            .find(|l| l.root.unwrap_or(false))
            .or_else(|| self.layouts.first())
    }

    /// The layout currently shown. Exactly one layout is active once the app is set up.
    pub fn get_active_layout(&self) -> Option<&Layout> {
        self.layouts.iter().find(|l| l.active)
    }

    pub fn get_active_layout_mut(&mut self) -> Option<&mut Layout> {
        self.layouts.iter_mut().find(|l| l.active)
    }

    /// Shows the layout with `layout_id`. An unknown id is rejected and changes nothing.
    pub fn set_active_layout(&mut self, layout_id: &str) -> Result<(), String> {
        if self.get_layout_by_id(layout_id).is_none() {
            return Err(format!("no layout with id '{}'", layout_id));
        }
        for layout in &mut self.layouts {
            layout.active = layout.id == layout_id;
        }
        Ok(())
    }

    pub fn get_muxbox_by_id(&self, id: &str) -> Option<&MuxBox> {
        self.layouts
            .iter()
            .find_map(|layout| layout.get_muxbox_by_id(id))
    }

    pub fn get_muxbox_by_id_mut(&mut self, id: &str) -> Option<&mut MuxBox> {
        self.layouts
            .iter_mut()
            .find_map(|layout| layout.get_muxbox_by_id_mut(id))
    }

    /// The layout that contains the box `box_id`.
    pub fn layout_of_box(&self, box_id: &str) -> Option<&Layout> {
        self.layouts
            .iter()
            .find(|layout| layout.get_muxbox_by_id(box_id).is_some())
    }

    pub fn calculate_bounds(&self) -> HashMap<String, HashMap<String, Bounds>> {
        self.layouts
            .iter()
            .map(|layout| (layout.id.clone(), layout_bounds(layout, &self.screen)))
            .collect()
    }

    /// The bounds of the boxes of the shown layout, by box id.
    pub fn active_bounds(&mut self) -> HashMap<String, Bounds> {
        let Some(layout_id) = self.get_active_layout().map(|l| l.id.clone()) else {
            return HashMap::new();
        };
        self.get_adjusted_bounds(None)
            .get(&layout_id)
            .cloned()
            .unwrap_or_default()
    }

    /// How often the script of `box_id` runs: its own `refresh_interval`, else the nearest
    /// enclosing box's, else its layout's. `None` runs it once.
    pub fn refresh_interval(&self, box_id: &str) -> Option<u64> {
        let layout = self.layout_of_box(box_id)?;
        let muxbox = layout.get_muxbox_by_id(box_id)?;
        muxbox
            .refresh_interval
            .or_else(|| {
                layout
                    .ancestors_of(box_id)?
                    .iter()
                    .find_map(|ancestor| ancestor.refresh_interval)
            })
            .or(layout.refresh_interval)
    }

    /// Replaces the box with the same id as `muxbox`, keeping its place in the tree.
    /// Returns whether a box was replaced.
    pub fn replace_muxbox(&mut self, muxbox: MuxBox) -> bool {
        let replaced = self
            .layouts
            .iter_mut()
            .any(|layout| layout.replace_muxbox_recursive(&muxbox));
        if replaced {
            self.invalidate_layout_cache();
        }
        replaced
    }

    /// Prepares a validated configuration for display: records each box's parent, builds the
    /// tabs of every box, focuses a box in each layout and shows the root layout.
    pub fn setup(&mut self) {
        fn prepare(muxbox: &mut MuxBox, layout_id: &str, parent_id: Option<&str>) {
            muxbox.parent_layout_id = Some(layout_id.to_string());
            muxbox.parent_id = parent_id.map(str::to_string);
            muxbox.initialize_streams();
            let id = muxbox.id.clone();
            for child in muxbox.children.iter_mut().flatten() {
                prepare(child, layout_id, Some(&id));
            }
        }

        let root_id = self.get_root_layout().map(|layout| layout.id.clone());
        for layout in &mut self.layouts {
            let layout_id = layout.id.clone();
            for muxbox in layout.children.iter_mut().flatten() {
                prepare(muxbox, &layout_id, None);
            }
            // The configuration may name the focused box with `selected: true`; otherwise the
            // first box in focus order gets focus.
            let configured = layout
                .get_all_muxboxes()
                .iter()
                .find(|muxbox| muxbox.selected == Some(true))
                .map(|muxbox| muxbox.id.clone());
            let focus = configured.or_else(|| {
                layout
                    .get_muxboxes_in_tab_order()
                    .first()
                    .map(|muxbox| muxbox.id.clone())
            });
            match focus {
                Some(id) => layout.select_only_muxbox(&id),
                None => layout.deselect_all_muxboxes(),
            }
            layout.active = Some(&layout.id) == root_id.as_ref();
        }
        self.invalidate_layout_cache();
    }
}

#[derive(Debug, Clone)]
pub struct AppContext {
    pub app: App,
    pub config: Config,
    /// The loaded plugins, shared by every copy of the context.
    pub plugins: Arc<crate::plugin::Plugins>,
    /// The configuration file the app was loaded from; box moves and resizes are saved to it.
    pub yaml_file_path: Option<PathBuf>,
}

impl AppContext {
    pub fn new(app: App, config: Config) -> Self {
        AppContext {
            app,
            config,
            plugins: Arc::new(crate::plugin::Plugins::default()),
            yaml_file_path: None,
        }
    }

    pub fn with_yaml_path(mut self, path: PathBuf) -> Self {
        self.yaml_file_path = Some(path);
        self
    }

    /// Loads the plugins the app lists. Relative paths are resolved against the directory of
    /// the configuration file. Fails when a plugin cannot be loaded or a box names a component
    /// no plugin offers.
    pub fn load_plugins(&mut self) -> Result<(), String> {
        let base = self
            .yaml_file_path
            .as_deref()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .unwrap_or_default();
        let paths: Vec<PathBuf> = self
            .app
            .plugins
            .iter()
            .flatten()
            .map(|spec| base.join(&spec.path))
            .collect();
        let plugins = crate::plugin::Plugins::load(&paths)?;
        let missing: Vec<String> = self
            .app
            .layouts
            .iter()
            .flat_map(|layout| layout.get_all_muxboxes())
            .filter_map(|muxbox| {
                let component = muxbox.plugin_component.as_ref()?;
                (!plugins.offers(component)).then(|| {
                    format!(
                        "box '{}' uses plugin component '{}', which no loaded plugin offers",
                        muxbox.id, component
                    )
                })
            })
            .collect();
        if !missing.is_empty() {
            return Err(missing.join("\n"));
        }
        self.plugins = Arc::new(plugins);
        Ok(())
    }
}

/// Loads, validates and prepares the configuration in `path`.
///
/// Variables are substituted first, so `${...}` may be used in any string value; the result is
/// then checked against the schema, deserialized, and checked for consistency (unique ids,
/// references to existing boxes, layouts and choices).
pub fn load_app_from_yaml(path: &Path) -> Result<App, String> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read '{}': {}", path.display(), e))?;
    load_app_from_str(&contents, &path.display().to_string())
}

/// Like [`load_app_from_yaml`], for configuration text named `source` in error messages.
pub fn load_app_from_str(contents: &str, source: &str) -> Result<App, String> {
    let mut document: serde_yaml::Value =
        serde_yaml::from_str(contents).map_err(|e| format_yaml_error(&e, contents, source))?;
    document
        .apply_merge()
        .map_err(|e| format!("{}: invalid merge key: {}", source, e))?;

    crate::model::variables::substitute_document(&mut document, &|name| std::env::var(name).ok())
        .map_err(|e| format!("{}: {}", source, e))?;

    let json = serde_json::to_value(&document).map_err(|e| {
        format!(
            "{}: configuration cannot be represented as data: {}",
            source, e
        )
    })?;
    validate_schema(&json).map_err(|errors| {
        let lines: Vec<String> = errors.iter().map(|e| format!("  {}", e)).collect();
        format!("{}: invalid configuration:\n{}", source, lines.join("\n"))
    })?;

    let root: TemplateRoot = serde_yaml::from_value(document)
        .map_err(|e| format!("{}: invalid configuration: {}", source, e))?;
    let mut app = root.app;
    validate_semantics(&app).map_err(|errors| {
        let lines: Vec<String> = errors.iter().map(|e| format!("  {}", e)).collect();
        format!("{}: invalid configuration:\n{}", source, lines.join("\n"))
    })?;
    app.setup();
    Ok(app)
}

/// A YAML syntax error with the offending line of `contents` and a caret under the column.
fn format_yaml_error(error: &serde_yaml::Error, contents: &str, source: &str) -> String {
    let Some(location) = error.location() else {
        return format!("{}: {}", source, error);
    };
    let (line, column) = (location.line(), location.column());
    let mut message = format!("{}:{}:{}: {}", source, line, column, error);
    if let Some(text) = contents.lines().nth(line.saturating_sub(1)) {
        let number = line.to_string();
        message.push_str(&format!(
            "\n{} | {}\n{} | {}^",
            number,
            text,
            " ".repeat(number.len()),
            " ".repeat(column.saturating_sub(1))
        ));
    }
    message
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load(yaml: &str) -> Result<App, String> {
        load_app_from_str(yaml, "test.yaml")
    }

    const TWO_LAYOUTS: &str = r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: left
          position: {x1: "0%", y1: "0%", x2: "50%", y2: "100%"}
          tab_order: 2
          content: "left"
        - id: right
          position: {x1: "50%", y1: "0%", x2: "100%", y2: "100%"}
          tab_order: 1
          content: "right"
    - id: other
      children:
        - id: solo
          position: {x1: "0%", y1: "0%", x2: "100%", y2: "100%"}
          content: "solo"
"#;

    #[test]
    fn root_layout_is_active_and_lowest_tab_order_box_is_focused() {
        let app = load(TWO_LAYOUTS).unwrap();
        assert_eq!(app.get_active_layout().unwrap().id, "main");
        let focused: Vec<&str> = app
            .get_active_layout()
            .unwrap()
            .get_selected_muxboxes()
            .iter()
            .map(|b| b.id.as_str())
            .collect();
        assert_eq!(focused, vec!["right"]);
    }

    #[test]
    fn switching_to_an_unknown_layout_is_rejected_and_keeps_the_current_one() {
        let mut app = load(TWO_LAYOUTS).unwrap();
        assert!(app.set_active_layout("nope").is_err());
        assert_eq!(app.get_active_layout().unwrap().id, "main");
        app.set_active_layout("other").unwrap();
        assert_eq!(app.get_active_layout().unwrap().id, "other");
        assert_eq!(app.layouts.iter().filter(|l| l.active).count(), 1);
    }

    #[test]
    fn variables_resolve_innermost_first_and_feed_validated_fields() {
        let app = load(
            r#"
app:
  variables: {COLOR: red, WHO: app}
  layouts:
    - id: main
      variables: {WHO: layout}
      children:
        - id: outer
          variables: {WHO: outer}
          position: {x1: "0", y1: "0", x2: "100%", y2: "100%"}
          border_color: "${COLOR}"
          title: "${WHO}"
          children:
            - id: inner
              variables: {WHO: inner}
              position: {x1: "0", y1: "0", x2: "50%", y2: "50%"}
              title: "${WHO} in ${MISSING:nowhere}"
              script: "for i in 1 2; do echo ${i}; done"
"#,
        )
        .unwrap();
        let outer = app.get_muxbox_by_id("outer").unwrap();
        assert_eq!(
            outer.border_color,
            Some(crate::model::appearance::ColorName::Red)
        );
        assert_eq!(outer.title.as_deref(), Some("outer"));
        let inner = app.get_muxbox_by_id("inner").unwrap();
        assert_eq!(inner.title.as_deref(), Some("inner in nowhere"));
        assert_eq!(
            inner.script.as_deref(),
            Some(&["for i in 1 2; do echo ${i}; done".to_string()][..])
        );
    }

    #[test]
    fn unknown_fields_are_rejected_with_their_location() {
        let err = load(
            r#"
app:
  layouts:
    - id: main
      children:
        - id: a
          position: {x1: "0", y1: "0", x2: "10", y2: "10"}
          pty: true
"#,
        )
        .unwrap_err();
        assert!(err.contains("pty"), "{}", err);
    }

    #[test]
    fn dangling_references_are_rejected() {
        let err = load(
            r#"
app:
  hot_keys: {F1: nope}
  layouts:
    - id: main
      children:
        - id: a
          position: {x1: "0", y1: "0", x2: "10", y2: "10"}
          redirect_output: missing_box
"#,
        )
        .unwrap_err();
        assert!(err.contains("missing_box"), "{}", err);
        assert!(err.contains("nope"), "{}", err);
    }

    #[test]
    fn focus_cannot_jump_to_a_box_of_another_layout() {
        let err = load(
            r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: a
          position: {x1: "0", y1: "0", x2: "10", y2: "10"}
          next_focus_id: b
    - id: other
      children:
        - id: b
          position: {x1: "0", y1: "0", x2: "10", y2: "10"}
"#,
        )
        .unwrap_err();
        assert!(
            err.contains("next_focus_id") && err.contains("other"),
            "{}",
            err
        );
    }

    #[test]
    fn yaml_syntax_errors_name_the_line() {
        let err = load("app:\n  layouts:\n    - id: [unclosed\n").unwrap_err();
        assert!(err.contains("test.yaml"), "{}", err);
        assert!(err.contains('3') || err.contains('4'), "{}", err);
    }

    #[test]
    fn tabs_are_built_and_first_choice_selected_for_nested_boxes() {
        let app = load(
            r#"
app:
  layouts:
    - id: main
      children:
        - id: parent
          position: {x1: "0", y1: "0", x2: "100%", y2: "100%"}
          children:
            - id: menu
              position: {x1: "0", y1: "0", x2: "50%", y2: "50%"}
              choices:
                - id: one
                  content: One
                - id: two
                  content: Two
"#,
        )
        .unwrap();
        let menu = app.get_muxbox_by_id("menu").unwrap();
        assert_eq!(menu.parent_id.as_deref(), Some("parent"));
        assert_eq!(menu.selected_choice_index(), Some(0));
        assert_eq!(menu.get_tab_labels(), vec!["Choices"]);
    }
}
