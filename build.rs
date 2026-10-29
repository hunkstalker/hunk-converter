use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=assets/branding/icon.ico");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_env == "gnu"
        && !command_exists("x86_64-w64-mingw32-windres")
        && !command_exists("windres")
    {
        println!("cargo:warning=windres no disponible; se omite el icono del ejecutable");
        return;
    }

    let mut resource = winresource::WindowsResource::new();
    resource.set_icon("assets/branding/icon.ico");
    resource.set("ProductName", "HUNK Converter");
    resource.set("FileDescription", "HUNK Converter · Image Toolbox");
    resource.set("CompanyName", "HUNK Converter contributors");
    resource.set("LegalCopyright", "MIT");
    if let Err(error) = resource.compile() {
        if target_env == "msvc" {
            panic!("no se pudo incrustar el icono de Windows: {error}");
        }
        println!("cargo:warning=no se pudo incrustar el icono de Windows: {error}");
    }
}

fn command_exists(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}
