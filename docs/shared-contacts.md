# ContactCraft's shared contacts

ContactCraft keeps its contacts where every Craft app on the computer can use them, **whether
ContactCraft is running or not**. This page is for the people building those apps
(MessageCraft, MailCraft and the next ones).

## Where

```
%APPDATA%\ContactCraft\           (or the folder in the CONTACTCRAFT_DATA environment variable)
├── contacts.db                   SQLite 3 database, WAL mode
├── photos\                       contact pictures   (<contact id>-<hash>.png|jpg|gif|webp)
├── backgrounds\                  theme banner and background pictures
├── fonts\                        font files a theme uses that aren't Windows' own
└── contactcraft-ui.json          ContactCraft's own window settings (not shared data)
```

Several programs can have `contacts.db` open at the same time (SQLite's WAL mode with a busy
timeout). Nothing runs in the background and there's no server: each app opens the file itself.

For a ready-made brief to give an AI coding agent working on MessageCraft (or MailCraft), see
[messagecraft-ai-instructions.md](messagecraft-ai-instructions.md).

## From Rust: use the `contactcraft-store` crate

```toml
# Use the release with the group API:
contactcraft-store = { git = "https://github.com/william-pyle/ContactCraft", tag = "v1.2.0" }
# Or use a local checkout: contactcraft-store = { path = "../ContactCraft/store" }
```

```rust
use contactcraft_store::{Store, Contact};

let store = Store::open_default()?;                  // %APPDATA%\ContactCraft

// Look someone up the way a message or an email arrives.
if let Some(c) = store.find_by_phone("+1 (555) 010-2030")? {   // any format; last 10 digits match
    let name = c.display_name();
    let picture = store.photo_path(&c);              // Option<PathBuf>
    let accent = &c.theme.accent;                    // "#0B6FB8" or "" for "pick from the name"
    let bubble = &c.theme.bubble_in;                 // their message colour in a conversation
    let font = store.font_path(&c);                  // the theme's font file, if any
    let background = store.background_path(&c);
}
let by_mail = store.find_by_email("anna@example.com")?;
let everyone = store.list()?;                        // sorted the way they file
let hits = store.search("smith")?;

// Groups can contain contact IDs and bare phone numbers.
use contactcraft_store::{ContactGroup, GroupMember};
let mut group = ContactGroup::new("Weekend team");
group.add_contact("<saved-contact-id>");
group.add_number("+1 555 010 9999");
store.save_group(&mut group)?;                       // stable ID and timestamps
let groups = store.list_groups()?;
let same_group = store.get_group(&group.id)?;
group.remove_member(&GroupMember::Number { number: "+1 555 010 9999".into() });
store.save_group(&mut group)?;
store.delete_group(&group.id)?;

// Change or add one: ContactCraft shows it within a couple of seconds.
let mut c = Contact::new();
c.name.given = "Sam".into();
c.phones.push(contactcraft_store::Labeled::new("mobile", "+1 555 010 9999"));
store.save(&mut c)?;                                 // gives it an id and timestamps
store.delete(&c.id)?;

// Keep a copy in memory and refresh only when something changed.
let rev = store.revision();
// ... later:
if store.revision() != rev {
    for (contact, deleted) in store.changes_since(rev)? { /* update your copy */ }
    // Reload list_groups() to see group additions, edits and deletions.
}
```

Everything else a contact holds is on `Contact` (see `store/src/model.rs`): name parts and phonetic
names, organisations, every phone/email/address/website/social profile/chat/SIP entry with its
label, dates, relationships, gender and pronouns, languages, hobbies and interests, IDs, desks
and buildings, calendar links, public keys, Outlook's own fields, custom fields, labels, notes,
favourite, picture and theme.

### Groups

`ContactGroup` has `id`, `name`, `theme`, `members`, `created`, `updated` and an `extra` map for
unknown fields. `GroupMember` is either `Contact { id }` or `Number { number }` (JSON:
`{"kind":"contact","id":"..."}` or `{"kind":"number","number":"+1 ..."}`). Group names
must be nonempty. The group theme is the same `ContactTheme` used for a person; an untouched
`ContactTheme::default()` means App Default. A number-only member remains valid without a saved
contact. Deleting a contact changes its group memberships to number-only entries for each phone
number it had.

`Store::save_group`, `delete_group`, `get_group` and `list_groups` all use the shared database.
Writes bump `Store::revision()` so other open apps can reload contacts and groups. For incremental
group additions and edits, `groups_changed_since(rev)` is available; reload `list_groups()` to
detect deletions. `theme_background_path(&group.theme)`, `theme_font_path(&group.theme)` and
`banner_path(&group.theme)` resolve theme files for a group.

**Fields you add.** If your app wants to keep something of its own on a contact, add it as a top
level JSON key with your app's name in it (for example `"messagecraftLastTexted"`). ContactCraft
keeps keys it doesn't know (`Contact::extra`), shows them under "From other apps", and never drops
them.

### The theme

| Field | Meaning |
| --- | --- |
| `accent`, `accent2` | Main and second colour (`#RRGGBB`); empty accent: pick a colour from the name |
| `style` | Banner pattern: `solid`, `gradient`, `diagonal`, `stripes`, `dots`, `glow` |
| `bannerImage` | Optional banner picture in `backgrounds\`; use `Store::banner_path(&theme)` |
| `text` | Text colour on the banner; empty: white or black, whichever reads better |
| `avatarShape` | `circle`, `rounded` or `square` |
| `emoji` | One emoji for the person |
| `font`, `fontFile` | Font family name and its file (`Store::font_path` finds it) |
| `textSize` | Points; `0` means App Default |
| `bold`, `italic` | Text style |
| `textColor`, `linkColor` | Text and link colours on the background |
| `background` | Background colour behind their card and conversation |
| `backgroundImage`, `backgroundFit`, `backgroundOpacity`, `backgroundBlur` | Background picture (`Store::background_path`), `cover`/`contain`/`tile`/`center`/`stretch`, strength 0–100, soften |
| `bubbleIn`, `bubbleInText`, `bubbleOut`, `bubbleOutText`, `bubbleRadius` | Message bubble colours in a conversation and their corner roundness (`0`: App Default) |
| `ringtone`, `textTone` | Sound name or file for calls and messages |

An empty colour always means App Default. `Store::set_banner(&mut theme, owner_id, bytes)` writes
PNG, JPEG, GIF or WebP bytes to the shared theme folder; save the contact or group afterwards.

## Asking ContactCraft to edit a contact

Any app can start `ContactCraft.exe --new-contact`, `--edit-contact <id>`, `--edit-phone <number>`
or `--edit-email <address>` to show only ContactCraft's editor, in a window that stays on top.
It returns exit code 0 (saved, with `saved <id>` on standard output), 1 (cancelled) or 2 (error),
and `--result-file <path>` writes the outcome as JSON. `Store::contactcraft_exe()` finds the
program. See the README's "Command line for other apps".

## From other languages: the database

`contacts.db` has two data tables plus `meta`. Existing version 1 databases gain the group table
when opened by the current crate; the contact table remains compatible.

```sql
CREATE TABLE contacts (
  id TEXT PRIMARY KEY,          -- 32 hex digits
  display_name TEXT,            -- what to show
  sort_key TEXT,                -- lower-case, how it files
  emails TEXT,                  -- ' a@x.com b@y.com '  (lower-case, space-wrapped)
  phones TEXT,                  -- ' 5550102030 ... '   (last 10 digits of each number)
  favorite INTEGER,
  photo TEXT,                   -- file name in photos\
  theme_accent TEXT,
  google_resource TEXT,         -- external sync resource, when present
  dirty INTEGER,                -- changed locally since external sync
  deleted INTEGER,              -- 1: deleted (skip these rows)
  rev INTEGER,                  -- the revision of its last change
  created TEXT, updated TEXT,   -- RFC 3339, UTC
  data TEXT                     -- the whole contact as JSON (camelCase keys, see model.rs)
);
CREATE TABLE groups (
  id TEXT PRIMARY KEY,          -- 32 hex digits
  name TEXT NOT NULL,
  rev INTEGER NOT NULL,         -- revision of its last change
  data TEXT NOT NULL            -- ContactGroup JSON, including members and theme
);
CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT);   -- 'schema' = 2, 'revision' = change counter
```

Reading: `SELECT data FROM contacts WHERE deleted = 0 AND phones LIKE '% 5550102030 %'`.

Writing from another language: in one transaction, add 1 to `meta.revision`, then insert or
replace the contact or group row with `rev` set to the new revision and `data` holding the full
JSON (keep keys you don't know). For contacts, fill the other index columns from the JSON as
above. The Rust crate does this; using it is the safe way.

## Files

Import and export go through vCard 3.0 (Google, Outlook, Apple), Google CSV, Outlook CSV and
ContactCraft's own JSON (`{"format": "contactcraft", "version": 2, "contacts": [...], "groups": [...]}`).
Version 1 JSON imports remain supported. Contact/group theme file names are stored in JSON, but
the files themselves are not embedded; back up the whole data folder to preserve pictures and
fonts. vCard and CSV import/export remain contact-only.
