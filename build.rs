fn main() {
    // Only embed the icon on Windows
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        // Check if icon file exists and try to compile, but don't fail the build
        if std::path::Path::new("resources/icon.ico").exists() {
            res.set_icon("resources/icon.ico");
            if let Err(e) = res.compile() {
                println!("cargo:warning=Failed to compile Windows resources: {}. Continuing without icon.", e);
            }
        } else {
            println!("cargo:warning=Icon file not found. Continuing without embedded icon.");
        }
    }
}
