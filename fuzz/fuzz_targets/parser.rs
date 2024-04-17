#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|smith_module: wasm_smith::Module| {
    let data = smith_module.to_bytes();
    let parser = wasmcat_parser::module::Module::from_bytes(&data);
    let _module = match parser.parse() {
        Ok(v) => v,
        Err(err) => panic!("{}", err),
    };
});
