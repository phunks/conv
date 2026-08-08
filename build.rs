fn main() {
    println!("cargo:rerun-if-changed=assets/windows/conv.ico");

    #[cfg(target_os = "windows")]
    {
        let mut resources = winres::WindowsResource::new();
        resources.set_icon("assets/windows/conv.ico");
        resources.compile().expect("failed to embed Windows resources");
    }
}