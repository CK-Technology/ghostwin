fn main() {
    if let Err(e) = slint_build::compile("ui/ghostwin.slint") {
        eprintln!("Failed to compile Slint UI: {}", e);
        eprintln!("Make sure ui/ghostwin.slint exists and is valid");
        std::process::exit(1);
    }
}