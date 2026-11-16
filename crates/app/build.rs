fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../../assets/app_icon.ico");
        if let Err(err) = res.compile() {
            println!("cargo:warning=No se pudo incrustar el icono de Windows: {err}");
        }
    }
}
