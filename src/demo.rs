//! Made-up contacts for `ContactCraft.exe --demo` (kept in a temporary folder, never the real
//! store).

use contactcraft_store::{Address, Contact, ContactTheme, CustomField, Email, Labeled, Organization};

use crate::app::App;

pub fn fill(app: &mut App) {
    let Some(store) = &app.store else { return };
    if !store.list().map(|l| l.is_empty()).unwrap_or(false) {
        return;
    }
    let mut people: Vec<Contact> = Vec::new();
    let mut c = Contact::new();
    c.name.given = "Avery".into();
    c.name.family = "Lindqvist".into();
    c.name.nickname = "Ave".into();
    c.pronouns = "she/her".into();
    c.favorite = true;
    c.phones = vec![Labeled::new("mobile", "+1 555 010 2030"), Labeled::new("work", "+1 555 010 4400")];
    c.emails = vec![Email { label: "home".into(), address: "avery@example.com".into(), display_name: String::new() }];
    c.organizations = vec![Organization { label: "work".into(), company: "Northwind Studio".into(), title: "Art Director".into(), department: "Design".into(), current: true, ..Default::default() }];
    c.addresses = vec![Address { label: "home".into(), street: "12 Harbor Lane".into(), city: "Portland".into(), region: "OR".into(), postal_code: "97201".into(), country: "United States".into(), ..Default::default() }];
    c.birthday = "1988-04-12".into();
    c.relations = vec![Labeled::new("partner", "Sam Okafor")];
    c.labels = vec!["Friends".into(), "Book club".into()];
    c.hobbies = vec!["pottery".into(), "trail running".into()];
    c.notes = "Prefers texts to calls. Allergic to peanuts.".into();
    c.theme = ContactTheme { accent: "#0B6FB8".into(), accent2: "#29B6D8".into(), style: "glow".into(), emoji: "🌊".into(), background: "#EEF6FB".into(), font: "Georgia".into(), font_file: "georgia.ttf".into(), ..Default::default() };
    people.push(c);

    let mut c = Contact::new();
    c.name.prefix = "Dr.".into();
    c.name.given = "Rafael".into();
    c.name.family = "Moreno".into();
    c.phones = vec![Labeled::new("work", "+1 555 013 7788"), Labeled::new("work fax", "+1 555 013 7789")];
    c.emails = vec![Email { label: "work".into(), address: "r.moreno@clinic.example".into(), display_name: "Dr. Moreno".into() }];
    c.organizations = vec![Organization { label: "work".into(), company: "Riverside Family Clinic".into(), title: "Physician".into(), office: "Suite 210".into(), current: true, ..Default::default() }];
    c.outlook.customer_id = "RFC-20931".into();
    c.outlook.billing_information = "Insurance on file".into();
    c.labels = vec!["Health".into()];
    c.theme = ContactTheme { accent: "#2E7D4F".into(), accent2: "#9CCB6B".into(), style: "stripes".into(), ..Default::default() };
    people.push(c);

    let mut c = Contact::new();
    c.name.given = "Mira".into();
    c.name.family = "Chen".into();
    c.favorite = true;
    c.phones = vec![Labeled::new("mobile", "+1 555 018 2211")];
    c.emails = vec![Email { label: "home".into(), address: "mira.chen@example.net".into(), display_name: String::new() }];
    c.birthday = "--09-30".into();
    c.labels = vec!["Family".into()];
    c.relations = vec![Labeled::new("sister", "Lena Chen")];
    c.custom_fields = vec![CustomField { label: "Shirt size".into(), value: "M".into(), kind: "text".into() }];
    c.theme = ContactTheme { accent: "#8E2C7A".into(), accent2: "#E05C9A".into(), style: "dots".into(), emoji: "🎉".into(), bubble_in: "#F6DDEB".into(), ..Default::default() };
    people.push(c);

    let mut c = Contact::new();
    c.kind = "organization".into();
    c.organizations = vec![Organization { label: "work".into(), company: "Greenleaf Plumbing".into(), current: true, ..Default::default() }];
    c.phones = vec![Labeled::new("main", "+1 555 019 0000")];
    c.websites = vec![Labeled::new("work", "greenleaf.example")];
    c.labels = vec!["Home services".into()];
    people.push(c);

    let mut c = Contact::new();
    c.name.given = "Jonah".into();
    c.name.family = "Whitfield".into();
    c.phones = vec![Labeled::new("mobile", "+44 7700 900123")];
    c.time_zone = "Europe/London".into();
    c.languages = vec!["English".into(), "Welsh".into()];
    c.labels = vec!["Friends".into()];
    people.push(c);

    // Messy and repeated entries, for trying Clean Up.
    let mut c = Contact::new();
    c.name.given = "AVERY".into();
    c.name.family = "lindqvist**".into();
    c.phones = vec![Labeled::new("mobile", "5550102030"), Labeled::new("other", "+1 555.010.2030")];
    c.emails = vec![Email { label: "work".into(), address: "avery@northwind.example".into(), display_name: String::new() }];
    c.birthday = "1988-04-21".into();
    c.addresses = vec![Address { label: "work".into(), street: "400 PINE STREET, SUITE 12".into(), city: "PORTLAND".into(), region: "oregon".into(), postal_code: "972041234".into(), country: "usa".into(), ..Default::default() }];
    people.push(c);
    let mut c = Contact::new();
    c.name.given = "Jonah".into();
    c.name.family = "Whitfield".into();
    c.phones = vec![Labeled::new("mobile", "+44 7700 900123")];
    c.time_zone = "Europe/London".into();
    c.languages = vec!["English".into(), "Welsh".into()];
    c.labels = vec!["Friends".into()];
    people.push(c);

    for mut p in people {
        let _ = store.save(&mut p);
    }
}
