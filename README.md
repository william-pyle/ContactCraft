# ContactSolutions

**Your contacts and contact groups on Windows, with details, pictures and themes shared with
other applications.**

ContactSolutions is a Windows contacts manager in the Craft family's style (the look of MailCraft and
MessageCraft). It keeps everything Google Contacts, Outlook, Apple Contacts and vCard can hold
about a person, plus anything else you want to add. It also keeps a picture and a theme for each
person. Integrating applications can read and change the same contacts directly.
Existing MailCraft and MessageCraft versions require separate migrations to the renamed store.

**1.3.0 is unreleased.** The latest published release is ContactCraft v1.2.4; its historical
asset names remain unchanged. The ContactSolutions package names below describe the next build.

[**Download the latest published release**](https://github.com/william-pyle/ContactSolutions/releases/latest)

---

## Contents

- [Features](#features)
- [Installing](#installing)
- [Getting started](#getting-started)
- [Importing your contacts](#importing-your-contacts)
- [Editing a contact](#editing-a-contact)
- [Contact groups](#contact-groups)
- [Themes](#themes)
- [Clean Up and duplicates](#clean-up-and-duplicates)
- [Exporting](#exporting)
- [Sharing with Craft apps](#sharing-with-craft-apps)
- [Command line for other apps](#command-line-for-other-apps)
- [Updates](#updates)
- [Where your data is kept](#where-your-data-is-kept)
- [Keyboard](#keyboard)
- [Options and command line](#options-and-command-line)
- [Building from source](#building-from-source)
- [Making a release](#making-a-release)
- [Project layout](#project-layout)
- [Troubleshooting](#troubleshooting)
- [Licence](#licence)

## Features

- **Every detail about a person**
  - Name in all its parts: prefix, first, middle, last, suffix, nickname, maiden name, phonetic
    (Yomi) spellings, initials, "file as" and a display name.
  - As many phone numbers, emails, addresses, websites, social profiles, chat accounts and SIP
    addresses as you need, each with a standard label (mobile, work, home fax, pager,
    assistant...) or any label of your own.
  - Work: several organisations, each with company, department, job title, office, job
    description, phonetic company, stock symbol, cost centre, website domain and dates.
  - Dates: birthday (with or without the year), anniversary, any other dates, birthplace, and date
    and place of death.
  - Relationships (spouse, partner, child, parent, manager, assistant, referred by...), gender,
    pronouns, languages, time zone, map location, hobbies, interests, skills and expertise.
  - IDs (account, customer, employee, government...), desks and buildings, calendar links and
    public keys.
  - All of Outlook's own fields: account, customer ID, government ID, organisational ID, billing
    information, mileage, directory server, computer network name, FTP site, free/busy address,
    subject, keywords, user fields 1–4, priority, sensitivity, private, journal and follow-up flag.
  - **Your own fields** at any time: give a field a name and a kind (text, number, date, link,
    email, phone or yes/no).
  - Labels (Google's contact groups, Outlook's categories), notes and favourites.
  - Anything another program writes that ContactSolutions doesn't know is kept as it is and shown
    under "From other apps", so nothing is ever lost.
- **A picture** for each contact: choose one or drop it on the window. Shown round, rounded or
  square.
- **A theme** for each contact, with a live preview: banner colours, pattern or picture, emoji, font,
  text size, bold or italic, text and link colours, a background colour or picture (fit,
  strength, soften), message bubble colours and roundness, and sounds. Eight ready-made looks to
  start from.
- **Contact groups** with a name, a standard theme and any mix of saved contacts and phone
  numbers. Other applications can add groups through the shared store.
- **Import** from Google Contacts (vCard or Google CSV), Outlook (CSV), iPhone or Mac (vCard), or
  ContactSolutions's own file. It can skip contacts you already have and label what came in.
- **Export** to vCard, Google CSV or ContactSolutions's own file.
- **Clean Up**: names in proper case without stray symbols, phone numbers and addresses in one
  standard format, repeated emails and numbers removed, and duplicate contacts found and merged.
  A backup is saved first, and the clean-up can be undone.
- **Shared with Craft apps**: one database every Craft app can open, whether ContactSolutions is
  running or not.
- **Updates**: checks GitHub for a new release when it starts, and downloads and installs it if
  you agree.
- **The Craft look**: title bar with search, File / Home / View / Help ribbon, rail, label pane,
  list and card, status bar; light, dark or follow-Windows, zoom and tighter spacing.

## Installing

1. Download **`ContactSolutions-windows-x64.msi`** from the
   [latest release](https://github.com/william-pyle/ContactSolutions/releases/latest) and run it.
   Like the other Craft apps' installers, it installs into Program Files, adds a Start Menu
   shortcut, and offers to launch ContactSolutions when it finishes.
2. ContactSolutions isn't code-signed yet, so Windows SmartScreen may warn the first time. Choose
   **More info › Run anyway**.

*Portable instead:* download `ContactSolutions-windows.zip`, unzip it anywhere and run
`ContactSolutions.exe`. Both versions use the same contacts.

To remove ContactSolutions, use Windows **Settings › Apps**. Your contacts stay in
`%APPDATA%\ContactSolutions` until you delete that folder yourself.

Requirements: Windows 10 or 11, 64-bit.

## Getting started

Open **ContactSolutions** from the Start menu.

- **New Contact** (or Ctrl+N) opens the editor on a new contact.
- **Import** brings in a file from Google Contacts or Outlook (see below).
- Click a contact in the list to see its card. **Edit** changes its details; **Theme** changes
  how it looks.
- The left pane filters the list by favourites, recent changes, themes, labels or groups. Select a
  group to manage its members and theme.
- The search box at the top searches names, companies, emails, phone numbers (in any format),
  addresses, labels, relationships, your own fields and notes.

To look around first with made-up contacts (nothing is saved), run `ContactSolutions.exe --demo`.

The top toolbar also controls the window: click the app icon for the Windows system menu,
drag empty space to move, or double-click empty space to maximize/restore. Resize using any
edge or corner. Minimize, maximize/restore and close are at the right. Alt+Space opens the
window menu and Alt+F4 closes the window. These controls also appear in editor-only dialogs;
closing a dialog cancels it. At narrow widths, Sharing remains available as an icon.

## Importing your contacts

Click **Import** (or drop a file on the window). ContactSolutions shows how many contacts the file
holds and a few names. You can skip contacts you already have (same email, phone or name) and
give everything that comes in a label. Columns ContactSolutions has no field for become your own
fields, so nothing in the file is dropped.

**From Google Contacts**

1. Open [contacts.google.com](https://contacts.google.com) and sign in.
2. Select the contacts to bring (or none, for all), then choose **Export**.
3. Pick **vCard** (keeps pictures) or **Google CSV**, then **Export**.
4. In ContactSolutions, click **Import** and choose the downloaded file.

**From Outlook**

- Classic Outlook: **File › Open & Export › Import/Export › Export to a file › Comma Separated
  Values**, then choose **Contacts**.
- New Outlook or Outlook.com: **People › Manage contacts › Export contacts**.
- Import the `.csv` file. Outlook's own fields (account, billing information, user fields...)
  are kept.

**From an iPhone or Mac:** in Contacts, select the contacts, then **File › Export › Export
vCard**, and import the `.vcf` file. Pictures come along.

## Editing a contact

**Edit** opens every field, in sections you can fold away: Name, Picture, Phone/email/chat,
Addresses, Work, Web and social, Dates, Relationships, Personal, IDs/places/calendars/keys,
Outlook details, Your own fields, and Labels and notes. Every list grows with **Add...** and each
entry has a label menu with the standard labels plus room for your own. **Save** (Ctrl+S) stores
the contact; ContactSolutions asks before throwing away unsaved changes.

## Contact groups

Click **New Group** on the Home ribbon or **New group** in the left pane. Give the group a name,
then use **Add contact** or **Add number**. Remove a member with **Remove**; this does not delete
the contact. A number-only member does not need a matching ContactSolutions contact. Group changes are
saved as you make them, and other applications can see them through the shared store. Deleting a
contact keeps its phone numbers in its groups as number-only members.
Labels imported with contacts remain labels; they do not automatically create group records.

Groups use the same theme fields as contacts. Choose a preset or set banner colours, a pattern or
picture, background, text, message colours and sounds. **App Default** restores the standard theme.

## Themes

**Theme** opens the theme editor beside a live preview of the person's card and a short
conversation:

| Part | What you can set |
| --- | --- |
| Banner | Main and second colour, pattern (gradient, solid, diagonal, stripes, dots, glow), optional picture, banner text colour |
| Picture | Shape (circle, rounded, square), an emoji |
| Text | Font (any Windows font, or a font file of your own), size, bold, italic, text colour, link colour |
| Background | Colour, and/or a picture with fit (cover, contain, tile, centre, stretch), strength and soften |
| Messages | Their bubble and text colours, yours, and how round the bubbles are (MessageCraft uses these) |
| Sounds | A ringtone and a message sound |

Start from one of the ready-made looks (Ocean, Sunset, Forest, Berry, Midnight, Slate, Gold,
Plain), or **Reset theme** to go back to the plain look. Fonts, banner pictures and
background pictures you add are copied into ContactSolutions's data folder, so every Craft app can use them.

## Clean Up and duplicates

**Home › Clean Up** tidies your contacts. Every step can be switched off, and it can run on all
contacts or only the ones shown:

1. **Names in proper case**: `JOHN o'brien` → `John O'Brien`, `mcdonald` → `McDonald`,
   `van der berg` → `van der Berg`, `iii` → `III`. Names already written in mixed case (DeShawn,
   MacKenzie) are left as they are.
2. **Special characters removed from names**: symbols, numbers and emoji go. Letters of any
   alphabet, accents, hyphens and apostrophes stay.
3. **Phone numbers in one standard format**: `5550102030` → `(555) 010-2030`, `1-555-010-2030 x45`
   → `+1 (555) 010-2030 ext. 45`, `+44 (0)7700-900123` → `+44 7700 900123`. Short codes and
   lettered numbers are left alone.
4. **Addresses in one standard format**: tidy spacing and capitals, standard street words (St,
   Ave, Rd, Blvd, Apt, Ste...), US states as their two-letter codes, ZIP+4 as `12345-6789`,
   Canadian and UK postcodes, and one name (and code) per country.
5. **Repeated email addresses** on a contact removed (capitals don't matter).
6. **Repeated phone numbers** on a contact removed (the same number written two ways counts as
   one).
7. **Duplicate contacts** found by the same name (in any order or capitals) or a phone number in
   common:
   - **Exact copies** are removed straight away, keeping the oldest.
   - The others open the **merge screen**, one group at a time. Pick the entry to keep (the most
     complete one is picked for you), choose which entry's value wins for each field they disagree
     on, and **Merge into one**. Phone numbers, emails, addresses, labels and every other list are
     combined from all entries without repeats. **Keep all** remembers that they're different
     people, so they aren't suggested again.

Before anything changes, a copy of every contact and picture is saved in
`%APPDATA%\ContactSolutions\backups`. **Undo the last clean-up** (in the Clean Up window) puts every
contact back as it was. Contacts added since stay.

## Exporting

**Export** (or File › Export) saves all contacts, or only the ones shown, as:

- **vCard (.vcf)**: for Google Contacts, Outlook, iPhone and almost everything else. Keeps
  pictures, and carries every ContactSolutions field and theme for another ContactSolutions.
- **Google CSV (.csv)**: the spreadsheet Google Contacts imports.
- **ContactSolutions file (.json)**: contact and group records, including group membership and theme
  settings. The file does not embed picture or font files; back up the data folder to keep those.
  Exporting only the contacts currently shown omits groups.

## Sharing with Craft apps

Your contacts live in one SQLite database in your Windows profile. Several programs can have it
open at once, and nothing has to be running. A change one app makes shows in the others within a
couple of seconds. Contact groups live in the same store, with contacts and number-only members.
Each contact carries its picture and theme, so a person looks the same in every app.

- For developers: [docs/shared-contacts.md](docs/shared-contacts.md) describes the
  `contactsolutions-store` Rust crate (in `store/`) and the database layout.
- [docs/messagecraft-ai-instructions.md](docs/messagecraft-ai-instructions.md) is a ready-made
  brief to give an AI coding agent so it connects MessageCraft (or MailCraft) to ContactSolutions's
  contacts: finding, showing, changing and adding them, and using their pictures and themes.

## Command line for other apps

Other apps can ask ContactSolutions to create or edit one contact or group. ContactSolutions then opens a
small window that stays on top of other windows and shows only the editor. **Save** stores the
record and closes the window; **Cancel** or closing the window closes it without saving.

```text
ContactSolutions.exe --new-contact [--name "Anna Smith"] [--given Anna] [--family Smith]
                 [--phone "+1 555 010 2030"] [--email anna@example.com] [--company Northwind]
                 [--label Friends] [--notes "Met at the conference"]
ContactSolutions.exe --edit-contact <id>
ContactSolutions.exe --edit-phone "<number>"     edits the contact with that number (any format),
                                             or starts a new one with it filled in
ContactSolutions.exe --edit-email <address>      the same, by email address
ContactSolutions.exe --open-group <id>            edit an existing group in a dialog
ContactSolutions.exe --new-group --group-name "Weekend team"
                 [--group-number "+1 555 010 2030" ...]
Add to any of them:
  --result-file "<path>"   write the outcome as JSON
For contacts:
  --theme-tab              open on the theme editor instead of the details
```

How the calling app learns what happened:

| | Saved | Cancelled or closed | Not found / failed |
| --- | --- | --- | --- |
| Exit code | `0` | `1` | `2` |
| Standard output | `saved <id>` | `cancelled` | `error <message>` |
| `--result-file` | `{"status": "saved", "id": "...", "displayName": "..."}` | `{"status": "cancelled"}` | `{"status": "error", "message": "..."}` |

To find `ContactSolutions.exe`, use `Store::contactsolutions_exe()` in the `contactsolutions-store` crate.
ContactSolutions records its own location in the shared store each time it starts, and the standard
install folder (`C:\Program Files\ContactSolutions`) is tried too. A running ContactSolutions window
shows the change within a couple of seconds.
## Updates

When ContactSolutions starts, it asks GitHub for the latest release of this repository. If it's newer
than the copy you're running, ContactSolutions shows what's new and offers to:

- **Download and install**: it downloads the installer with a progress bar, then **Install now**
  closes ContactSolutions and runs the installer, which replaces the old version. Your contacts aren't
  touched.
- **Later**: it asks again next time (a link in the status bar brings the prompt back).
- **Skip this version**: it doesn't ask about that version again.

Check by hand with **Help › Check for Updates**. Turn the start-up check off in **File › Options
› Updates**. The check is a single anonymous request for the public release list: nothing about
you or your contacts is sent.

## Where your data is kept

Shared data is in `%APPDATA%\ContactSolutions`, or the folder named by
`CONTACTSOLUTIONS_DATA`. UI settings always stay in `%APPDATA%\ContactSolutions`,
independent of that store override:

| Path | What it is |
| --- | --- |
| `contacts.db` | Contacts and groups (SQLite, shared with other applications) |
| `photos\` | Contact pictures |
| `backgrounds\` | Theme banner and background pictures |
| `fonts\` | Font files themes use that aren't Windows' own |
| `backups\` | A copy of every contact and picture from before each clean-up |
| `contactsolutions-ui.json` | ContactSolutions's own window settings |

See [rename migration and recovery](docs/rename-migration.md) before the first renamed-app startup.
Back up both the data folder and settings profile when using a custom store. **File › Info › Open the folder** opens it.

## Keyboard

| Keys | Does |
| --- | --- |
| Ctrl+N | New contact |
| Ctrl+F or Ctrl+E | Search |
| Ctrl+S | Save the contact being edited |
| Enter | Edit the selected contact; confirm a pending delete dialog |
| Delete | Delete the selected contact (asks first) |
| ↑ / ↓ | Move through the list |
| Esc | Close the editor or the File view |

## Options and command line

**File › Options**: appearance (follow Windows, light, dark), zoom, the label pane, tighter
spacing (one line per contact), sort by last name, and the update check.

| Argument | What it does |
| --- | --- |
| `--demo` | Made-up contacts in a temporary folder, removed when ContactSolutions closes |
| `--light` / `--dark` | Start in that appearance |
| `--edit` / `--theme` | (demo) open the first contact in the editor or theme editor |
| `--cleanup` / `--cleanup-run` | (demo) open Clean Up, or run it straight away |
| `--no-update-check` | Don't check for a new version this time |
| `--new-contact`, `--edit-contact`, `--edit-phone`, `--edit-email` | Open only the editor for one contact (see [Command line for other apps](#command-line-for-other-apps)) |
| `--open-group <id>`, `--new-group --group-name <name> [--group-number <number> ...]` | Open only the group editor; the group is saved when Save is pressed |

## Building from source

Requirements:

- Windows 10/11, x64
- [Rust](https://rustup.rs) (stable, MSVC toolchain) with the Visual Studio C++ build tools
  (SQLite is compiled in)
- For the installer: [WiX v5](https://wixtoolset.org)
  (`dotnet tool install --global wix --version 5.0.2`). Without WiX the build makes everything
  except the MSI.
- Node.js is optional: it only gives the `npm run build` shortcut.

```powershell
npm run build
# or, without Node:
powershell -ExecutionPolicy Bypass -File build.ps1
```

`build.ps1` runs the workspace tests (including window hit testing), builds `ContactSolutions.exe` in release mode, and writes to
`dist\`:

- `ContactSolutions.exe`
- `ContactSolutions-windows-x64.msi`: the installer (`packaging/windows/package.ps1` with
  `contactsolutions.wxs`)
- `ContactSolutions-windows.zip`: the portable version
- `index.html`, `help.html`: the download page
- the guides and notices

Tests on their own: `cargo test --workspace`.

Native window smoke checks: `powershell -ExecutionPolicy Bypass -File scripts/check-window-chrome.ps1`.
This launches isolated debug/demo fixtures for the main window and both editor dialogs,
checks native geometry and window state, saves screenshots and verifies settings and dialog
cancellation. It does not drive real mouse/keyboard input. Manually verify toolbar dragging,
edge/corner dragging, icon/right-click/Alt+Space menus, Alt+F4, snapping, and moving between
monitors with different DPI. At minimum size with high app zoom, the existing content panes
can overflow; the window toolbar stays available.

The installer takes its version from `Cargo.toml`, so the program, the update check and the
installer always agree. Its UpgradeCode is ContactSolutions's own: a newer MSI replaces an older one
in place.

## Making a release

1. Change `version` in `Cargo.toml` and `store/Cargo.toml` (for example to `1.3.0`) and update
   `packaging/release-notes.md`.
2. Commit, then tag and push:

   ```powershell
   git tag v1.3.0
   git push origin main v1.3.0
   ```

3. The **Build** workflow (`.github/workflows/build.yml`) builds and tests on Windows, makes the
   installer and the zip, and publishes a GitHub Release with both attached. The tag must match
   the version in `Cargo.toml`.
4. Copies of ContactSolutions that are already installed see the new release the next time they
   start, and offer to install it.

Every push to `main` and every pull request is built as well; the files are kept as a workflow
artifact.

## Project layout

```
ContactSolutions/
├── src/                     the window (Rust, egui/eframe)
│   ├── app.rs               state, settings, loading, pictures and fonts
│   ├── chrome.rs            title bar, ribbon, rail, status bar, dialogs
│   ├── list.rs              label pane and contact list
│   ├── card.rs              the contact card and theme drawing
│   ├── editor.rs            the full editor
│   ├── theme_editor.rs      the contact theme editor and preview
│   ├── groups.rs            group membership and theme editor
│   ├── cleanup_ui.rs        Clean Up, the merge screen, backups and undo
│   ├── io.rs                import and export
│   ├── backstage.rs         the File view
│   ├── update.rs            the update check, download and install
│   ├── dialog.rs            contact and group editor windows other apps open
│   └── theme.rs, icons.rs, widgets.rs   the Craft look
├── store/                   contactsolutions-store: the shared contact library
│   ├── src/model.rs         contacts, groups and their themes
│   ├── src/store.rs         the SQLite store, groups, pictures and theme files
│   ├── src/vcard.rs         vCard in and out
│   ├── src/csv.rs           Google CSV and Outlook CSV in, Google CSV out
│   ├── src/cleanup.rs       clean-up rules, duplicates and merging
│   └── tests/               store, import and clean-up tests
├── docs/                    the sharing guide and the MessageCraft AI brief
├── packaging/               the installer (WiX), icon script, release notes
├── web/                     download and help pages
└── build.ps1                the build
```

## Troubleshooting

- **SmartScreen warns about the installer.** ContactSolutions isn't code-signed yet; choose
  **More info › Run anyway**.
- **An import found no contacts.** Export from Google as *Google CSV* or *vCard*, or from Outlook
  as a *CSV* file. A file exported as *Outlook CSV* from Google also works.
- **A theme font shows App Default instead.** The font file isn't on this computer: add it
  with **Another font file…** in the theme editor, so it's copied into ContactSolutions's data
  folder.
- **The update check says it couldn't check.** The computer is offline, or GitHub couldn't be
  reached; try **Help › Check for Updates** later, or download from the Releases page.
- **A clean-up changed something you wanted.** Open Clean Up and choose **Undo the last
  clean-up**.
- **Another Craft app's change doesn't show.** It appears within about two seconds; **File › Info
  › Read again** reloads everything.

## Licence

MIT. See [LICENSE](LICENSE) and [NOTICE.md](NOTICE.md). The look follows MailCraft's design, as
MessageCraft does. Inter typeface: SIL Open Font License 1.1.
