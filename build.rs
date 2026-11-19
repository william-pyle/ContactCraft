// Puts ContactSolutions's icon and version information into ContactSolutions.exe on Windows.
fn main() {
    println!("cargo:rerun-if-changed=assets/contactsolutions.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/contactsolutions.ico")
            .set("ProductName", "ContactSolutions")
            .set("FileDescription", "ContactSolutions")
            .set("CompanyName", "ContactSolutions contributors")
            .set("LegalCopyright", "ContactSolutions contributors. MIT OR Apache-2.0");
        if let Err(e) = res.compile() {
            println!("cargo:warning=couldn't add the icon to ContactSolutions.exe: {e}");
        }
    }
}
