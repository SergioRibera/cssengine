use std::sync::{OnceLock, RwLock};

use cssengine::StyleSheet;

static THEME: OnceLock<RwLock<StyleSheet>> = OnceLock::new();

fn theme() -> &'static RwLock<StyleSheet> {
    THEME.get_or_init(|| RwLock::new(StyleSheet::default()))
}

/// Initialize the global CSS theme from a string.
///
/// Call once at app startup. Subsequent calls replace the stylesheet.
pub fn init(css: &str) {
    match THEME.get() {
        Some(lock) => {
            *lock.write().expect("cssengine theme lock poisoned") = StyleSheet::from_css(css);
        }
        None => {
            let _ = THEME.set(RwLock::new(StyleSheet::from_css(css)));
        }
    }
}

/// Return the declarations for `selector`, filtering out pseudo-class rules.
pub(crate) fn base_decls(selector: &str) -> Vec<cssengine::Declaration> {
    let mut ss = theme().write().expect("cssengine theme lock poisoned");
    ss.get_styles(selector)
        .into_iter()
        .filter(|(pseudo, _)| pseudo.is_none())
        .flat_map(|(_, decls)| decls)
        .collect()
}

// ---------------------------------------------------------------------------
// File-watching hot-reload (non-WASM only)
// ---------------------------------------------------------------------------

/// Watch `path` for changes and hot-reload the stylesheet automatically.
///
/// Spawns a background thread; returns immediately.  Safe to call before or
/// after [`init`].
#[cfg(not(target_arch = "wasm32"))]
pub fn watch(path: &'static str) {
    use std::sync::mpsc;
    use std::thread;

    thread::spawn(move || {
        use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

        let (tx, rx) = mpsc::channel::<()>();

        let mut watcher = match RecommendedWatcher::new(
            move |res: notify::Result<notify::Event>| {
                if res.is_ok() {
                    let _ = tx.send(());
                }
            },
            Config::default(),
        ) {
            Ok(w) => w,
            Err(_) => return,
        };

        if watcher
            .watch(std::path::Path::new(path), RecursiveMode::NonRecursive)
            .is_err()
        {
            return;
        }

        // Block forever: each file-change event triggers a reload.
        // `watcher` stays in scope here so it keeps watching.
        for () in rx {
            if let Ok(css) = std::fs::read_to_string(path) {
                init(&css);
            }
        }
        drop(watcher);
    });
}

#[cfg(target_arch = "wasm32")]
pub fn watch(_path: &'static str) {}
