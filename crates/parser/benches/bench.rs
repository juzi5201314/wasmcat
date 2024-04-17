#![feature(test)]

use test::{black_box, Bencher};

extern crate test;

macro_rules! bench_wasm {
    ($size_hint:ident: $path:literal) => {
        paste::paste! {
            #[bench]
            pub fn [<$size_hint _wasm>](b: &mut Bencher) {
                let data = include_bytes!($path);
                b.bytes = data.len() as u64;
                b.iter(|| {
                    let parser = wasmcat_parser::module::Module::from_bytes(data);
                    let _ = black_box(parser.parse());
                });
            }

            #[cfg(feature = "parallel")]
            #[bench]
            pub fn [<$size_hint _wasm_parallel>](b: &mut Bencher) {
                let data = include_bytes!($path);
                b.bytes = data.len() as u64;
                b.iter(|| {
                    let parser = wasmcat_parser::module::Module::from_bytes(data);
                    let _ = black_box(parser.par_parse());
                });
            }
        }
    };
}

bench_wasm!(small_30kb: "../tests/lots-of-types.wasm");
bench_wasm!(medium_180kb: "../tests/bz2.wasm");
bench_wasm!(big_1900kb: "../tests/pulldown-cmark.wasm");
bench_wasm!(very_big_5270kb: "../tests/spidermonkey.wasm");