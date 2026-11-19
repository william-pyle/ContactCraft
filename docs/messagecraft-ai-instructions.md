# Instructions for an AI coding agent: connect MessageCraft to ContactSolutions

Paste everything below the line into the coding agent (Claude Code, Codex, Cursor...) working in
the MessageCraft project. The same steps work for MailCraft:
replace "MessageCraft" with "MailCraft" and "conversation" with "email".

---

## Goal

MessageCraft must use **ContactSolutions's contacts** directly: find them, show them, change them and
add new ones, **whether ContactSolutions is running or not**. Use each contact's name, picture and
theme in conversations. Adapt the integration to the application's current People and conversation
code; verify those paths before changing them.

Existing MailCraft/MessageCraft builds are not compatible with this renamed interface.
Their migrations are separate work; this guide describes the target interface.
Settings remain in the roaming ContactSolutions profile even with a custom data directory.

## Where the contacts are

- One SQLite database, `%APPDATA%\ContactSolutions\contacts.db` (WAL mode), plus the folders
  `photos\`, `backgrounds\` and `fonts\` beside it. If the environment variable
  `CONTACTSOLUTIONS_DATA` is set, that folder is used instead.
- Nothing has to be running: open the file yourself. Several programs may have it open at once.
- **Don't write SQL against it.** Use the Rust crate that ContactSolutions itself uses. It keeps the
  index columns, the change counter and the JSON in step, and keeps fields MessageCraft doesn't
  know about.

## Add the crate

In MessageCraft's `Cargo.toml` under `[dependencies]`:

```toml
contactsolutions-store = { git = "https://github.com/william-pyle/ContactSolutions", tag = "v1.3.0" }
```

The renamed crate is version 1.3.0. For local development, use
`contactsolutions-store = { path = "../ContactSolutions/store" }` instead.
Historical versions retain their original crate name.

It brings `rusqlite` with SQLite bundled (compiled with the MSVC C compiler MessageCraft already
builds with), `serde`, `serde_json`, `chrono`, `base64`, `sha2` and `getrandom`. The crate's own
documentation is `store/src/lib.rs`; the full field list is `store/src/model.rs`; the database
layout is `docs/shared-contacts.md` in the same project.

## The API you need

```rust
use contactsolutions_store::{Store, Contact, Labeled, Email, phone_key};

let store = Store::open_default()?;            // opens (or creates) %APPDATA%\ContactSolutions
store.list()?                                  // Vec<Contact>, every contact, sorted
store.get(&id)?                                // Option<Contact>
store.find_by_phone("+1 (555) 010-2030")?      // any format; the last 10 digits are compared
store.find_by_email("anna@example.com")?       // case doesn't matter
store.search("smith")?                         // name, company, emails, phones, notes, labels...
store.save(&mut contact)?                      // add (gives it an id) or change
store.delete(&id)?
store.revision()                               // i64 that moves whenever any app changes a contact
store.changes_since(rev)?                      // Vec<(Contact, deleted: bool)> since that revision
store.photo_path(&c)                           // Option<PathBuf> of the contact's picture
store.set_photo(&mut c, &bytes)?               // store a picture (PNG/JPEG/GIF/WebP), then save(c)
store.background_path(&c)                      // the theme's background picture, if any
store.banner_path(&c.theme)                     // the theme's banner picture, if any
store.font_path(&c)                            // the theme's font file, if any
store.list_groups()?                            // groups of contacts and bare numbers
store.get_group(&id)?                          // Option<ContactGroup>
store.save_group(&mut group)?                   // add or update a group
store.delete_group(&id)?                       // remove a group, leaving contacts intact
```

Errors are `String`s (`Result<T, String>`).

`Contact` helpers: `display_name()`, `initials()`, `company()`, `job_title()`,
`phone_keys()`, `email_keys()`, `search_text()`. The free function `phone_key(number)` gives the
key two numbers share when they're the same number.

The crate also has `contactsolutions_store::cleanup` (`format_phone`, `proper_case`,
`dedupe_phones`...) if MessageCraft wants to show numbers the same way ContactSolutions does.

## How to build it into MessageCraft

1. **Threading.** `Store` holds a `rusqlite::Connection`: it is `Send` but not `Sync`. Open it
   in a worker thread and talk to it through channels. Never block the egui frame on it. Calls are fast (milliseconds)
   but the window must never wait on the disk.
2. **Load and keep current.** On start, load `store.list()` into memory and build a map from
   `phone_key` to contact id. Every 2 seconds, compare `store.revision()` with the revision you
   loaded. When it moved, apply `store.changes_since(old)` (drop the ones marked deleted) and
   rebuild the map. That's how edits made in ContactSolutions show up in MessageCraft.
3. **Names on conversations.** Wherever the phone only gives a number (`looks_like_number` in
   `people.rs`), look the number up through the map and show `display_name()`.
4. **Pictures.** Load `store.photo_path(&c)` with the `image` crate into an egui texture, cache
   it by the file name (names change when the picture changes), and draw it instead of initials.
   Honour `c.theme.avatar_shape` (`circle`, `rounded`, `square`). Without a picture, draw the
   initials on `c.theme.accent` (when it's empty, keep MessageCraft's own avatar colour).
5. **Theme in the conversation.** Colours are `#RRGGBB` strings; **an empty string always means
   "use MessageCraft's own"**.
   - Their bubbles: `theme.bubble_in`, text `theme.bubble_in_text`. Yours: `theme.bubble_out`,
     `theme.bubble_out_text`. Corner roundness `theme.bubble_radius` (0 = your own).
   - Behind the conversation: `theme.background` colour, and/or the picture at
     `store.background_path(&c)` drawn with `theme.background_fit` (`cover`, `contain`, `tile`,
     `center`, `stretch`) at `theme.background_opacity` percent, softened when
     `theme.background_blur` is true.
   - Text: `theme.font` / `store.font_path(&c)` (add the font file to egui's fonts as its own
     family, with MessageCraft's fonts after it for missing characters), `theme.text_size` in
     points (0 = your own), `theme.bold`, `theme.italic`, `theme.text_color`,
     `theme.link_color`.
   - The conversation header can use `theme.accent` / `theme.accent2` and `theme.style`
     (`solid`, `gradient`, `diagonal`, `stripes`, `dots`, `glow`), `theme.emoji` and the optional
     picture at `store.banner_path(&theme)`.
   - Sounds: `theme.text_tone` (a sound name or a file path) for their messages.
6. **The People view.** Point `people_list` and `people_card` at ContactSolutions's contacts instead
   of MailCraft's. Show at least name, picture, phones, emails and notes. Offer **Text** (the
   existing `text_number`), **Edit** and **New contact**. For Edit and New contact, start
   ContactSolutions's own editor window (see "Let ContactSolutions do the editing" below) rather than
   writing a new editor; the "add this number" button in a conversation can use
   `--edit-phone <number>`.
7. **Changing and adding contacts.** Always start from the full contact: `store.get(&id)`, change
   the fields you edit, then `store.save(&mut c)`. Never build a fresh `Contact` for an existing
   person: that would wipe the fields MessageCraft doesn't show. New contacts:
   `let mut c = Contact::new();` set `c.name.given` / `c.name.family`, push a phone
   `Labeled::new("mobile", number)`, then `store.save(&mut c)` (it fills in the id and times).
   The "add this number to contacts" button in a conversation does exactly that.
8. **Deleting.** `store.delete(&id)` after asking the person. It also removes the picture file.
9. **Your own fields.** To remember something MessageCraft-specific on a contact (for example
   when you last texted them), put it in `c.extra` under a key starting with `messagecraft`,
   such as `c.extra.insert("messagecraftLastTexted".into(), json!(timestamp))`. ContactSolutions
   keeps it and shows it under "From other apps".
10. **Groups.** Load `store.list_groups()` when the revision changes. A `ContactGroup` has a
    name, a `ContactTheme` and `members`: `GroupMember::Contact { id }` or
    `GroupMember::Number { number }`. Show number-only members even when no saved contact matches.
    Add or remove members through the group helpers and `store.save_group(&mut group)`; do not
    write group JSON directly. Deleting a group leaves its contacts alone.

## Let ContactSolutions do the editing (recommended)

Instead of building a full contact editor in MessageCraft, start ContactSolutions's own editor. It
opens as a small window that stays on top, showing only the editor, and closes when the person
saves or cancels:

```rust
use std::process::Command;
let store = contactsolutions_store::Store::open_default()?;
if let Some(exe) = store.contactsolutions_exe() {
    // Add the number from a conversation, or edit the contact that has it:
    let out = Command::new(&exe).args(["--edit-phone", &number]).output();   // run off the UI thread
    // New contact with details filled in:
    // Command::new(&exe).args(["--new-contact", "--name", "Sam Lee", "--phone", &number])
    // Edit a known contact:      .args(["--edit-contact", &contact.id])
    // Open on the theme editor:  add "--theme-tab"
    if let Ok(out) = out {
        match out.status.code() {
            Some(0) => { /* saved: stdout is "saved <id>"; reload or wait for revision() to move */ }
            Some(1) => { /* cancelled */ }
            _ => { /* not found or failed: stdout is "error <message>" */ }
        }
    }
}
```

Run it on a worker thread (it waits until the window closes). If `contactsolutions_exe()` returns
`None`, ContactSolutions isn't installed: hide the button, or fall back to MessageCraft's own editor.
`--result-file <path>` writes the same outcome as JSON when reading standard output is awkward.
## Don'ts

- Don't open `contacts.db` with raw SQL or copy it; use the crate.
- Don't delete or rename files in `photos\`, `backgrounds\` or `fonts\` yourself.
- Don't drop unknown fields: always save the `Contact` you got back from the store.
- Don't keep the `Store` on the UI thread or share one `Store` between threads.

## Checking your work

- Unit tests: point `CONTACTSOLUTIONS_DATA` at a temporary folder (or call
  `Store::open(&temp_dir)`) so tests never touch the real contacts. Save a contact with a phone
  number, find it by that number in another format, change it from a second `Store` opened on
  the same folder, and check the first one sees `revision()` move.
- By hand: run `ContactSolutions.exe --demo` to see sample contacts with themes (that demo keeps its
  own temporary store), or add a contact in ContactSolutions, give it a theme, and check that
  MessageCraft shows its name, picture, bubble colours and background on that person's
  conversation within a couple of seconds.
- Run `cargo build --release` and fix any warnings. Add a short "ContactSolutions contacts" section
  to MessageCraft's README describing the above.
