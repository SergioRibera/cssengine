//! # freya-css
//!
//! CSS styling integration for the [Freya](https://freyaui.dev/) UI framework,
//! powered by [`cssengine`].
//!
//! ## Quick start
//!
//! ```rust,ignore
//! use dioxus::prelude::*;
//! use freya::prelude::*;
//! use freya_css::{use_css_theme, CssClassExt};
//!
//! const CSS: &str = r#"
//!     .card {
//!         background-color: #ffffff;
//!         border-radius: 8px;
//!         padding: 16px;
//!     }
//!     .title {
//!         color: #1a1a1a;
//!         font-size: 20px;
//!         font-weight: 700;
//!     }
//! "#;
//!
//! fn app() -> Element {
//!     let _ = use_css_theme(CSS);
//!     rsx! {
//!         rect { width: "100%", height: "100%",
//!             {card_component()}
//!         }
//!     }
//! }
//!
//! fn card_component() -> Element {
//!     rsx! {
//!         rect { class: "card",
//!             label { class: "title", "freya-css" }
//!         }
//!     }
//! }
//! ```
//!
//! ## Hot-reload
//!
//! Replace [`use_css_theme`] with [`use_css_theme_file`] to watch a file on
//! disk and automatically re-apply styles when it changes:
//!
//! ```rust,ignore
//! let _ = use_css_theme_file("assets/style.css");
//! ```

mod context;
pub mod convert;
mod ext;
mod hooks;

pub use context::CssTheme;
pub use ext::CssClassExt;
pub use hooks::{use_css_theme, use_css_theme_file};
