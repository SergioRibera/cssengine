use cssengine::{Declaration, StyleSheet};

fn main() {
    let css = r#"
        button {
          border-radius: 5px;
        }
        .my-class {
            color: red;
            display: flex;
            flex-direction: column;
            background-color: blue;
        }

        #super-id {
          color: #323232;
        }

        .tailwindcss-test {
          display: grid;
          background-color: orange-200;
          color: slate-700;
        }
    "#;

    let mut stylesheet = StyleSheet::from_css(css);

    let classes = [
        "button",
        ".my-class",
        "#super-id",
        "#super-id .my-class",
        ".other-class",
        // Tailwindcss Colors
        ".bg-amber-50 .text-zinc-500",
        ".tailwindcss-test",
    ];

    for class in &classes {
        println!("{class:?}");
        for (_class, styles) in stylesheet.get_styles(class) {
            for style in styles {
                if let Declaration::BackgroundColor(c) = &style {
                    println!("\tBackgroundColor({})", c.to_hex_string());
                    continue;
                }
                if let Declaration::Color(c) = &style {
                    println!("\tColor({})", c.to_hex_string());
                    continue;
                }
                println!("\t{style:?}");
            }
        }
    }
}
