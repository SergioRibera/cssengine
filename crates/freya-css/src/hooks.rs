use cssengine::StyleSheet;
use dioxus::prelude::{use_context_provider, use_future};

use crate::context::CssTheme;

/// Parse a static CSS string into a [`CssTheme`] and make it available to
/// all descendant components via Dioxus context.
///
/// Call this **once** near the root of your component tree:
///
/// ```rust,ignore
/// fn App() -> Element {
///     let _ = use_css_theme(include_str!("../assets/style.css"));
///     rsx! { /* ... */ }
/// }
/// ```
pub fn use_css_theme(css: &'static str) -> CssTheme {
    use_context_provider(|| CssTheme::new(StyleSheet::from_css(css)))
}

/// Load a CSS file at `path`, parse it into a [`CssTheme`], and provide it to
/// all descendant components.
///
/// On non-WASM targets the file is watched with [`notify`] and hot-reloads
/// automatically when it changes on disk.
///
/// ```rust,ignore
/// fn App() -> Element {
///     let _ = use_css_theme_file("assets/style.css");
///     rsx! { /* ... */ }
/// }
/// ```
pub fn use_css_theme_file(path: &'static str) -> CssTheme {
    let mut theme = use_context_provider(|| {
        let css = std::fs::read_to_string(path).unwrap_or_default();
        CssTheme::new(StyleSheet::from_css(&css))
    });

    #[cfg(not(target_arch = "wasm32"))]
    use_future(move || async move {
        use tokio::sync::mpsc::unbounded_channel;

        let (tx, mut rx) = unbounded_channel::<()>();

        // Spin up a blocking thread for the `notify` watcher.
        // Only `tx` (which is `Send`) is moved into the thread.
        std::thread::spawn(move || {
            use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
            let (s_tx, s_rx) = std::sync::mpsc::channel();
            let Ok(mut watcher) = RecommendedWatcher::new(
                move |res: notify::Result<notify::Event>| {
                    if res.is_ok() {
                        let _ = s_tx.send(());
                    }
                },
                Config::default(),
            ) else {
                return;
            };
            let _ = watcher.watch(
                std::path::Path::new(path),
                RecursiveMode::NonRecursive,
            );
            // Keep the watcher alive until the sender is dropped.
            for _ in s_rx {
                let _ = tx.send(());
            }
        });

        // Receive change events and reload.
        while rx.recv().await.is_some() {
            if let Ok(css) = tokio::fs::read_to_string(path).await {
                theme.reload(&css);
            }
        }
    });

    theme
}
