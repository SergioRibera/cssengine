use cssengine::StyleSheet;
use divan::Bencher;

fn main() {
    divan::main();
}

#[divan::bench]
fn bench_single_instance(b: Bencher) {
    let input = "button { border-radius: 5px; }";

    b.bench(|| {
        _ = StyleSheet::from_css(input);
    })
}

#[divan::bench]
fn bench_iter_instance(b: Bencher) {
    b.with_inputs(||
        "img { width: 100%; display: block; border-radius: 0 0 1rem 1rem; } figure { margin: 0; } figcaption { padding: 0.5rem 1rem 0.4rem; background: #ddd; color: #333; border-radius: 1rem 1rem 0 0; text-align: end; }",
    )
    .bench_values(|input| {
        _ = StyleSheet::from_css(input);
    })
}

#[divan::bench]
fn bench_clone_instance(b: Bencher) {
    let input = ".my-class { color: red; background-color: blue; }";

    b.bench(|| {
        _ = StyleSheet::from_css(input);
    })
}
