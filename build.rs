// Puts ContactCraft's icon and version information into ContactCraft.exe on Windows.
fn main() {
    println!("cargo:rerun-if-changed=assets/contactcraft.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/contactcraft.ico")
            .set("ProductName", "ContactCraft")
            .set("FileDescription", "ContactCraft")
            .set("CompanyName", "ContactCraft contributors")
            .set("LegalCopyright", "ContactCraft contributors. MIT OR Apache-2.0");
        if let Err(e) = res.compile() {
            println!("cargo:warning=couldn't add the icon to ContactCraft.exe: {e}");
        }
    }
}
