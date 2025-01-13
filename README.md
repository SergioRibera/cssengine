# CSS Engine

CSS Engine is a lightweight, fast, and flexible CSS parser and style manager designed for Rust applications. It supports a wide range of features including animations, style sheets, and parsing CSS rules. The engine is modular, allowing users to enable or disable features like grid, flexbox, or Tailwind CSS color utilities based on their needs.

## Features

- **CSS Parsing**: Convert CSS strings into structured rules and declarations.
- **Style Sheet Management**: Efficiently manage styles with caching and rule matching.
- **Tailwind Color Support**: Dynamically generate styles for Tailwind-style color classes.
- **Animation Support**: Create animations with fine-grained control.
- **Feature Toggles**: Enable or disable specific functionalities like grid and flexbox layouts.
- **Integration Ready**: Built with performance and compatibility in mind.

Things I would like to have

- Improve error handling, it is currently horrible.
- Provide an abstraction through traits so that when using cssengine they can also provide an implementation for a virtual DOM.
- Once we have the Virtual DOM system, we can think of a Query system to search for classes and elements.
- Support more complex syntax
- Optimize the parser, I know it sounds bad, currently `cssengine` is **stupid fast**, but I know it can be more more more faster.
- Support more css values
- Eventually I would like to stop relying on `std`, nothing in particular, I don't think anyone would want to use css in embed, right? but I would like `std` to be a featue and not a requirement.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
cssengine = "0.1.0"
```

To enable additional features:

```toml
[features]
default = ["grid", "flexbox"]
grid = ["taffy/grid"]
flexbox = ["taffy/flexbox"]
tailwind_colors = []
serde = ["dep:serde", "smallvec/serde", "taffy/serde", "csscolorparser/serde"]
```

## Usage

### Parsing CSS

```rust
use cssengine::{StyleSheet, css_to_rules};

let css_input = "
    .example {
        background-color: red;
        color: white;
    }
";

let rules = css_to_rules(css_input).expect("Failed to parse CSS");
println!("{:?}", rules);
```

### Working with Style Sheets

```rust
use cssengine::StyleSheet;

let css_input = "
    .example {
        background-color: blue;
        color: white;
    }
";

let mut stylesheet = StyleSheet::from_css(css_input);
if let Some(styles) = stylesheet.get_styles(".example") {
    println!("Styles for .example: {:?}", styles);
}
```

### Tailwind Colors (Feature)

> [!IMPORTANT]
> Enable the `tailwind_colors` feature to use Tailwind-style color classes:

```rust
let mut stylesheet = StyleSheet::new_const();
if let Some(styles) = stylesheet.get_styles(".bg-blue-500 .text-red-600") {
    println!("Generated styles: {:?}", styles);
}
```

## Acknowledgments

- [`floem_css`](https://github.com/aalhitennf/floem-css): hell almost all of this engine is inspired by what they did in floem_css, an excellent project.
- [`csscolorparser`](https://github.com/mazznoer/csscolorparser-rs): saved me doing a lot of stuff for the parser and it's stupid light, which I love.
- [`taffy`](https://github.com/DioxusLabs/taffy): damn, what a great library, I love it, without it I wouldn't have done any of the layout support.
- [`smallvec`](https://github.com/servo/rust-smallvec): excellent library, super lightweight.
- [`divan`](https://github.com/nvzqz/divan): beautiful bookcase for the benches, thanks for making life easier.

- [`parse-color`](https://github.com/mintlu8/parse-color): I found it very useful to see how tailwindcss generation worked.

## License

CSS Engine is distributed under the MIT License. See [LICENSE](./LICENSE) for more details.
