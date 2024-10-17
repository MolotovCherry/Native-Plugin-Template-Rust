use std::{env, fs, path::Path};

fn main() {
    // stamp dll with project metadata
    let mut res = winres::WindowsResource::new();

    // allow high dpi scaling, compatibility, + modern visual style
    // the only reason this is here is because we want popups to look nice
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dir = Path::new(&manifest_dir);
    let manifest = fs::read_to_string(dir.join("manifest.xml")).unwrap();
    res.set_manifest(&manifest);

    let _ = res.compile();
}
