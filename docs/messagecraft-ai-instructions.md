# Instructions for an AI coding agent: connect MessageCraft to ContactCraft

Paste everything below the line into the coding agent (Claude Code, Codex, Cursor...) working in
the MessageCraft project. The same steps work for MailCraft:
replace "MessageCraft" with "MailCraft" and "conversation" with "email".

---

## Goal

MessageCraft must use **ContactCraft's contacts** directly: find them, show them, change them and
add new ones, **whether ContactCraft is running or not**. Use each contact's name, picture and
theme in conversations. This replaces (or comes before) the current MailCraft contacts link in
`src/mailcraft.rs` and `src/people.rs`; keep that link as a fallback only if ContactCraft has no
contacts.

## Where the contacts are

- One SQLite database, `%APPDATA%\ContactCraft\contacts.db` (WAL mode), plus the folders
  `photos\`, `backgrounds\` and `fonts\` beside it. If the environment variable
  `CONTACTCRAFT_DATA` is set, that folder is used instead.
- Nothing has to be running: open the file yourself. Several programs may have it open at once.
- **Don't write SQL against it.** Use the Rust crate that ContactCraft itself uses. It keeps the
  index columns, the change counter and the JSON in step, and keeps fields MessageCraft doesn't
  know about.

## Add the crate

In MessageCraft's `Cargo.toml` under `[dependencies]`:

```toml
contactcraft-store = { git = "https://github.com/william-pyle/ContactCraft", tag = "v1.0.0" }
```

(Use the newest release tag from https://github.com/william-pyle/ContactCraft/releases. If the
ContactCraft source is checked out on this computer, `{ path = "<that folder>/store" }` works too.)

```toml
```

It brings `rusqlite` with SQLite bundled (compiled with the MSVC C compiler MessageCraft already
builds with), `serde`, `serde_json`, `chrono`, `base64`, `sha2` and `getrandom`. The crate's own
documentation is `store/src/lib.rs`; the full field list is `store/src/model.rs`; the database
layout is `docs/shared-contacts.md` in the same project.

## The API you need

```rust
use contactcraft_store::{Store, Contact, Labeled, Email, phone_key};

let store = Store::open_default()?;            // opens (or creates) %APPDATA%\ContactCraft
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
store.font_path(&c)                            // the theme's font file, if any
```

Errors are `String`s (`Result<T, String>`).

`Contact` helpers: `display_name()`, `initials()`, `company()`, `job_title()`,
`phone_keys()`, `email_keys()`, `search_text()`. The free function `phone_key(number)` gives the
key two numbers share when they're the same number.

The crate also has `contactcraft_store::cleanup` (`format_phone`, `proper_case`,
`dedupe_phones`...) if MessageCraft wants to show numbers the same way ContactCraft does.

## How to build it into MessageCraft

1. **Threading.** `Store` holds a `rusqlite::Connection`: it is `Send` but not `Sync`. Open it
   in a worker thread and talk to it through channels, the way `src/mailcraft.rs` does with
   `Link`, `Request` and `Reply`. Never block the egui frame on it. Calls are fast (milliseconds)
   but the window must never wait on the disk.
2. **Load and keep current.** On start, load `store.list()` into memory and build a map from
   `phone_key` to contact id. Every 2 seconds, compare `store.revision()` with the revision you
   loaded. When it moved, apply `store.changes_since(old)` (drop the ones marked deleted) and
   rebuild the map. That's how edits made in ContactCraft show up in MessageCraft.
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
     (`solid`, `gradient`, `diagonal`, `stripes`, `dots`, `glow`), and `theme.emoji`.
   - Sounds: `theme.text_tone` (a sound name or a file path) for their messages.
6. **The People view.** Point `people_list` and `people_card` at ContactCraft's contacts instead
   of MailCraft's. Show at least name, picture, phones, emails and notes. Offer **Text** (the
   existing `text_number`), **Edit** and **New contact**, plus **Open in ContactCraft** (start
   `ContactCraft.exe` if installed; it shows the same contact).
7. **Changing and adding contacts.** Always start from the full contact: `store.get(&id)`, change
   the fields you edit, then `store.save(&mut c)`. Never build a fresh `Contact` for an existing
   person: that would wipe the fields MessageCraft doesn't show. New contacts:
   `let mut c = Contact::new();` set `c.name.given` / `c.name.family`, push a phone
   `Labeled::new("mobile", number)`, then `store.save(&mut c)` (it fills in the id and times).
   The "add this number to contacts" button in a conversation does exactly that.
8. **Deleting.** `store.delete(&id)` after asking the person. It also removes the picture file.
9. **Your own fields.** To remember something MessageCraft-specific on a contact (for example
   when you last texted them), put it in `c.extra` under a key starting with `messagecraft`,
   such as `c.extra.insert("messagecraftLastTexted".into(), json!(timestamp))`. ContactCraft
   keeps it and shows it under "From other apps".

## Don'ts

- Don't open `contacts.db` with raw SQL or copy it; use the crate.
- Don't delete or rename files in `photos\`, `backgrounds\` or `fonts\` yourself.
- Don't drop unknown fields: always save the `Contact` you got back from the store.
- Don't keep the `Store` on the UI thread or share one `Store` between threads.

## Checking your work

- Unit tests: point `CONTACTCRAFT_DATA` at a temporary folder (or call
  `Store::open(&temp_dir)`) so tests never touch the real contacts. Save a contact with a phone
  number, find it by that number in another format, change it from a second `Store` opened on
  the same folder, and check the first one sees `revision()` move.
- By hand: run `ContactCraft.exe --demo` to see sample contacts with themes (that demo keeps its
  own temporary store), or add a contact in ContactCraft, give it a theme, and check that
  MessageCraft shows its name, picture, bubble colours and background on that person's
  conversation within a couple of seconds.
- Run `cargo build --release` and fix any warnings. Add a short "ContactCraft contacts" section
  to MessageCraft's README describing the above.
