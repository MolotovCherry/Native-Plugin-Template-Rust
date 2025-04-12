static MANIFEST: &str = include_str!("manifest.xml");

fn main() {
    // stamp dll with project metadata
    let mut res = winres::WindowsResource::new();

    // allow high dpi scaling, compatibility, + modern visual style
    // the only reason this is here is because we want popups to look nice
    res.set_manifest(MANIFEST);

    let _ = res.compile();
}
