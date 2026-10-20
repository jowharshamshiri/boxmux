//! Builders for model objects used across tests.

use crate::model::app::{App, AppContext};
use crate::model::bounds::{Bounds, InputBounds};
use crate::model::layout::Layout;
use crate::model::muxbox::MuxBox;
use crate::Config;

pub struct TestDataFactory;

impl TestDataFactory {
    /// A box covering the top-left part of its parent with a little text content.
    pub fn create_test_muxbox(id: &str) -> MuxBox {
        MuxBox {
            id: id.to_string(),
            title: Some("Test MuxBox".to_string()),
            position: InputBounds::parse("0", "0", "100", "50").expect("valid position"),
            content: Some("Test content".to_string()),
            ..MuxBox::default()
        }
    }

    pub fn create_test_muxbox_with_parent(id: &str, parent_layout_id: &str) -> MuxBox {
        let mut muxbox = Self::create_test_muxbox(id);
        muxbox.parent_layout_id = Some(parent_layout_id.to_string());
        muxbox
    }

    /// A layout holding `muxboxes` (or one default box), each recording the layout as parent.
    pub fn create_test_layout(id: &str, muxboxes: Option<Vec<MuxBox>>) -> Layout {
        let mut boxes =
            muxboxes.unwrap_or_else(|| vec![Self::create_test_muxbox("default_muxbox")]);
        for muxbox in &mut boxes {
            muxbox.parent_layout_id = Some(id.to_string());
        }
        Layout {
            id: id.to_string(),
            title: Some(format!("Test Layout {}", id)),
            children: Some(boxes),
            root: Some(false),
            ..Layout::default()
        }
    }

    pub fn create_root_layout(id: &str, boxes: Option<Vec<MuxBox>>) -> Layout {
        let mut layout = Self::create_test_layout(id, boxes);
        layout.root = Some(true);
        layout
    }

    /// An app with one root layout, set up the way loading a configuration sets it up.
    pub fn create_test_app() -> App {
        let mut app = App::new();
        app.layouts = vec![Self::create_root_layout("test_layout", None)];
        app.setup();
        app
    }

    pub fn create_bounds(x1: usize, y1: usize, x2: usize, y2: usize) -> Bounds {
        Bounds { x1, y1, x2, y2 }
    }

    pub fn create_input_bounds(x1: &str, y1: &str, x2: &str, y2: &str) -> InputBounds {
        InputBounds::parse(x1, y1, x2, y2).expect("valid test position")
    }

    pub fn create_test_app_context() -> AppContext {
        AppContext::new(Self::create_test_app(), Config::default())
    }
}
