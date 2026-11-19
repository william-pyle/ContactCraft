"""Exercise v1.2.4 export code against current import/save code, using only synthetic data.

Builds an ignored test harness, not another checkout. The production import functions
are extracted verbatim; UI notification/reload state is supplied by a small test shell.
No install, APPDATA access, release, network request or owner data is involved.
"""
from pathlib import Path
import hashlib
import json
import os
import subprocess

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / ".tools" / "legacy-export-roundtrip"
FIXTURE.mkdir(parents=True, exist_ok=True)


def historical(path):
    return subprocess.check_output(["git", "show", f"v1.2.4:{path}"], cwd=ROOT)


paths = subprocess.check_output(["git", "ls-tree", "-r", "--name-only", "v1.2.4", "store"], cwd=ROOT, text=True).splitlines()
for path in paths:
    if path == "store/Cargo.toml" or path.startswith("store/src/"):
        target = FIXTURE / "legacy" / Path(path).relative_to("store")
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(historical(path))

old_io = historical("src/io.rs").decode("utf-8")
current_io = (ROOT / "src/io.rs").read_text(encoding="utf-8")
editor = (ROOT / "src/editor.rs").read_text(encoding="utf-8")
reader = current_io.split("pub fn read_contacts_file", 1)[1].split("\nimpl App {", 1)[0]
duplicate = current_io.split("    fn is_duplicate", 1)[1].split("\n    fn run_import", 1)[0]
importer = current_io.split("    fn run_import", 1)[1].split("\n    pub fn import_dialog", 1)[0]
shrink = editor.split("pub fn shrink_picture", 1)[1].split("\n// --- pieces", 1)[0]
json_expression = next(line for line in old_io.splitlines() if '"json" => serde_json::to_string_pretty' in line).strip().split(" => ", 1)[1].removesuffix(",").replace("self.groups.clone()", "groups.clone()")

(FIXTURE / "Cargo.toml").write_text('''[workspace]
[package]
name = "contactsolutions-export-check"
version = "0.0.0"
edition = "2024"
[dependencies]
legacy = { package = "contactcraft-store", path = "legacy" }
contactsolutions-store = { path = "../../store" }
serde_json = "1"
image = { version = "0.25", default-features = false, features = ["png"] }
''', encoding="utf-8")
(FIXTURE / "Cargo.lock").write_bytes((ROOT / "Cargo.lock").read_bytes())
(FIXTURE / "src").mkdir(exist_ok=True)
support = r'''
#![allow(dead_code)]
use std::{collections::{HashSet,HashMap}, path::{Path,PathBuf}};
use contactsolutions_store::{Contact,ContactGroup,GroupMember,Store,csv,vcard};
mod app { pub enum Filter { All, Label(String) } }
struct ImportPreview { items:Vec<(Contact,Option<Vec<u8>>)>, groups:Vec<ContactGroup>, skip_duplicates:bool, label:String }
struct ImportData { contacts:Vec<(Contact,Option<Vec<u8>>)>, groups:Vec<ContactGroup> }
struct App { import:Option<ImportPreview>, store:Option<Store>, contacts:Vec<Contact>, groups:Vec<ContactGroup>, filter:app::Filter, notices:Vec<String> }
impl App {
 fn new(path:&Path)->Self { Self { import:None,store:Some(Store::open(path).unwrap()),contacts:vec![],groups:vec![],filter:app::Filter::All,notices:vec![] } }
 fn reload(&mut self) { self.contacts=self.store.as_ref().unwrap().list().unwrap();self.groups=self.store.as_ref().unwrap().list_groups().unwrap(); }
 fn notify(&mut self,text:String,error:bool) { assert!(!error,"{text}");self.notices.push(text); }
 fn read_and_import(&mut self,path:&Path,skip:bool) { let data=read_contacts_file(path).unwrap();self.import=Some(ImportPreview{items:data.contacts,groups:data.groups,skip_duplicates:skip,label:String::new()});self.run_import(); }
}
'''
scenario = r'''
fn main() {
 let root=std::env::temp_dir().join(format!("contactsolutions-export-check-{}",contactsolutions_store::new_id()));
 std::fs::create_dir(&root).unwrap();
 let old=legacy::Store::open(&root.join("old-store")).unwrap();
 let mut contact=legacy::Contact::new();contact.name.given="Synthetic export fixture".into();
 contact.emails.push(legacy::Email{address:"fixture@example.invalid".into(),..Default::default()});
 contact.custom_fields.push(legacy::CustomField{label:"Fixture custom field".into(),value:"Retain this value".into(),kind:"text".into()});
 contact.extra.insert("future_field".into(),serde_json::json!({"retained":true}));
 contact.theme.accent="#123456".into();contact.theme.background_image="fixture-background.png".into();contact.theme.font_file="fixture-font.ttf".into();
 contact.google=Some(Default::default());
 let mut photo=std::io::Cursor::new(Vec::new());
 image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(4,4,image::Rgba([20,80,140,255]))).write_to(&mut photo,image::ImageFormat::Png).unwrap();
 let photo=photo.into_inner();old.set_photo(&mut contact,&photo).unwrap();old.save(&mut contact).unwrap();
 std::fs::create_dir_all(old.dir().join("backgrounds")).unwrap();std::fs::write(old.dir().join("backgrounds/fixture-background.png"),&photo).unwrap();
 std::fs::create_dir_all(old.dir().join("fonts")).unwrap();std::fs::write(old.dir().join("fonts/fixture-font.ttf"),b"synthetic-font-marker").unwrap();
 std::fs::write(old.dir().join("contactcraft-ui.json"),br#"{"zoom":150}"#).unwrap();
 let mut group=legacy::ContactGroup::new("Synthetic group");group.add_contact(&contact.id);group.add_number("+1 555 010 0999");old.save_group(&mut group).unwrap();
 let list=old.list().unwrap();let groups=old.list_groups().unwrap();let only_shown=false;
 let json=LEGACY_JSON_EXPRESSION;
 let vcf=legacy::vcard::export(&list,|c|old.photo_path(c).and_then(|p|std::fs::read(p).ok()));
 let json_path=root.join("old-export.json");let vcf_path=root.join("old-export.vcf");std::fs::write(&json_path,&json).unwrap();std::fs::write(&vcf_path,&vcf).unwrap();
 assert!(vcf.contains("X-CONTACTCRAFT-JSON"));
 let mut json_app=App::new(&root.join("json-import"));json_app.read_and_import(&json_path,true);
 assert_eq!(json_app.contacts.len(),1);assert_eq!(json_app.groups.len(),1);
 let imported=&json_app.contacts[0];assert_ne!(imported.id,contact.id);assert!(!imported.id.is_empty());
 assert_ne!(json_app.groups[0].id,group.id);assert!(json_app.groups[0].members.contains(&GroupMember::Contact{id:imported.id.clone()}));
 assert!(json_app.groups[0].members.contains(&GroupMember::Number{number:"+1 555 010 0999".into()}));
 assert_eq!(imported.custom_fields[0].value,"Retain this value");assert_eq!(imported.extra["future_field"]["retained"],true);
 assert!(imported.photo.is_empty());assert!(imported.google.is_none());
 assert_eq!(imported.theme.background_image,"fixture-background.png");assert_eq!(imported.theme.font_file,"fixture-font.ttf");
 assert!(!root.join("json-import/backgrounds/fixture-background.png").exists());assert!(!root.join("json-import/fonts/fixture-font.ttf").exists());
 assert!(!root.join("json-import/contactsolutions-ui.json").exists());
 let id=imported.id.clone();
 let mut modified=imported.clone();modified.custom_fields[0].value="Existing destination value".into();json_app.store.as_ref().unwrap().save(&mut modified).unwrap();json_app.reload();
 json_app.read_and_import(&json_path,true);assert_eq!(json_app.contacts.len(),1);assert_eq!(json_app.groups.len(),2);assert_eq!(json_app.contacts[0].id,id);
 assert_eq!(json_app.contacts[0].custom_fields[0].value,"Existing destination value");
 assert!(json_app.groups.iter().all(|g|g.members.contains(&GroupMember::Contact{id:id.clone()})));
 json_app.read_and_import(&json_path,false);assert_eq!(json_app.contacts.len(),2);assert_eq!(json_app.groups.len(),3);
 let mut vcf_app=App::new(&root.join("vcf-import"));vcf_app.read_and_import(&vcf_path,true);
 assert_eq!(vcf_app.contacts.len(),1);assert!(vcf_app.groups.is_empty());
 let imported=&vcf_app.contacts[0];assert_ne!(imported.id,contact.id);assert_eq!(imported.custom_fields[0].value,"Retain this value");assert_eq!(imported.extra["future_field"]["retained"],true);
 assert_eq!(std::fs::read(vcf_app.store.as_ref().unwrap().photo_path(imported).unwrap()).unwrap(),photo);
 assert_eq!(imported.theme.accent,contact.theme.accent);assert!(!root.join("vcf-import/backgrounds/fixture-background.png").exists());
 drop(old);drop(json_app);drop(vcf_app);
 println!("{}",serde_json::json!({"passed":true,"legacyVersion":"1.2.4","fixturePath":root,"json":{"contacts":1,"groups":1,"customFields":true,"unknownFields":true,"photoRestored":false,"newIds":true,"groupMembershipRemapped":true},"vcard":{"contacts":1,"groups":0,"photoRestored":true,"customFields":true,"newIds":true},"duplicateSkip":{"contactsAfterSecondImport":1,"groupsAfterSecondImport":2,"existingFieldsNotMerged":true},"skipDisabled":{"contactsAfterThirdImport":2,"groupsAfterThirdImport":3},"externalThemeFilesRestored":false,"settingsRestored":false}));
}
'''
source = support + "\npub fn read_contacts_file" + reader + "\nimpl App {\nfn is_duplicate" + duplicate + "\nfn run_import" + importer + "\n}\nmod editor { pub fn shrink_picture" + shrink + "\n}\n" + scenario.replace("LEGACY_JSON_EXPRESSION", json_expression)
(FIXTURE / "src/main.rs").write_text(source, encoding="utf-8")
env = dict(os.environ, CARGO_TARGET_DIR=str(ROOT / "target"))
result = subprocess.run(["cargo", "run", "--quiet", "--offline", "--manifest-path", str(FIXTURE / "Cargo.toml")], cwd=ROOT, env=env, capture_output=True, text=True)
if result.returncode:
    print(result.stdout)
    print(result.stderr)
    raise SystemExit(result.returncode)
report = json.loads(result.stdout.strip().splitlines()[-1])
report["legacyCommit"] = subprocess.check_output(["git", "rev-list", "-n", "1", "v1.2.4"], cwd=ROOT, text=True).strip()
report["currentIoSha256"] = hashlib.sha256((ROOT / "src/io.rs").read_bytes()).hexdigest()
report["legacyIoSha256"] = hashlib.sha256(historical("src/io.rs")).hexdigest()
report["scope"] = "Actual old exporter and current reader/save/duplicate functions in a synthetic harness; no UI clicks, install or owner data."
(FIXTURE / "result.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report, indent=2))
