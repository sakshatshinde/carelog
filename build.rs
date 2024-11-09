extern crate winres;

fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set("FileVersion", "0.0.1.0");
        res.set("ProductVersion", "0.1.0");
        res.set("CompanyName", "Sakshat Shinde");
        res.set("ProductName", "Carelog");
        res.set("LegalCopyright", "© Sakshat Shinde 2024");
        res.set("FileDescription", "Carelog helps save lives");
        res.set("OriginalFilename", "carelog.exe");
        res.set_icon("./assets/favicon.ico"); //default EGUI ico for now

        // Set language to U.S. English
        res.set_language(0x0409);
        res.compile().expect("Failed to compile Windows resources");
    }
}
