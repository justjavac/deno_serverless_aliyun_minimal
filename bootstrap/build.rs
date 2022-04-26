use deno_core::JsRuntime;
use deno_core::RuntimeOptions;

use std::env;
use std::path::PathBuf;

fn main() {
    let fc_extension = deno_core::Extension::builder()
        .js(deno_core::include_js_files!(
          prefix "deno:fc_runtime",
          "src/fc_runtime.js",
        ))
        .build();

    let o = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let snapshot_path = o.join("FC_RUNTIME_SNAPSHOT.bin");
    let options = RuntimeOptions {
        will_snapshot: true,
        extensions: vec![
            // deno_webidl::init(),
            // deno_console::init(),
            // deno_url::init(),
            // deno_web::init(BlobStore::default(), None),
            // deno_timers::init::<deno_timers::NoTimersPermission>(),
            // deno_webgpu::init(true),
            fc_extension,
        ],
        ..Default::default()
    };
    let mut isolate = JsRuntime::new(options);

    let snapshot = isolate.snapshot();
    let snapshot_slice: &[u8] = &*snapshot;
    println!("Snapshot size: {}", snapshot_slice.len());
    std::fs::write(&snapshot_path, snapshot_slice).unwrap();
    println!("Snapshot written to: {} ", snapshot_path.display());
}
