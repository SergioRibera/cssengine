//! # freya-css
//!
//! Global CSS styling for the [Freya](https://freyaui.dev/) UI framework,
//! powered by [`cssengine`].
//!
//! ## Quick start
//!
//! ```rust,ignore
//! use freya::prelude::*;
//! use freya_css::{init, CssClassExt};
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
//! fn main() {
//!     init(CSS);
//!     launch(app);
//! }
//!
//! fn app() -> Element {
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
//! ```rust,ignore
//! fn main() {
//!     freya_css::watch("assets/style.css");
//!     launch(app);
//! }
//! ```

pub mod convert;
mod ext;
mod theme;

pub use ext::CssClassExt;
pub use theme::{init, watch};
