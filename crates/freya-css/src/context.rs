use std::sync::{Arc, Mutex};

use cssengine::StyleSheet;
use dioxus::prelude::Signal;

/// A reactive CSS theme stored in Dioxus context.
///
/// The inner `Signal<Arc<Mutex<StyleSheet>>>` achieves two goals at once:
/// - **Mutation**: `get_styles` needs `&mut StyleSheet`; the `Mutex` grants that
///   without a write lock on the Signal.
/// - **Reactivity**: replacing the `Arc` in the Signal (on hot-reload) triggers
///   a re-render in every component that subscribed via `.class()`.
///
/// Obtain an instance with [`use_css_theme`] or [`use_css_theme_file`].
#[derive(Clone, Copy)]
pub struct CssTheme(pub Signal<Arc<Mutex<StyleSheet>>>);

impl CssTheme {
    pub(crate) fn new(sheet: StyleSheet) -> Self {
        CssTheme(Signal::new(Arc::new(Mutex::new(sheet))))
    }

    /// Replace the active stylesheet with a freshly-parsed one.
    ///
    /// Marks the Signal as changed so all subscribed components re-render.
    pub(crate) fn reload(&mut self, css: &str) {
        self.0.set(Arc::new(Mutex::new(StyleSheet::from_css(css))));
    }
}
