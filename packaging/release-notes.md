## ContactCraft 1.1.0

Your contacts on Windows, with every detail, a picture and a theme for each person, shared with
MessageCraft, MailCraft and every Craft app.

### New in 1.1.0

- **Other apps can open ContactCraft's editor.** MessageCraft, MailCraft or any program can start
  ContactCraft to add or change one contact. A small window that stays on top shows only the
  editor; it closes when you save or cancel, and tells the app what happened.
  - `ContactCraft.exe --new-contact` (with `--name`, `--phone`, `--email`, `--company`, `--label`
    or `--notes` filled in), `--edit-contact <id>`, `--edit-phone <number>` or
    `--edit-email <address>`.
  - Exit code 0 (saved), 1 (cancelled) or 2 (not found), a `saved <id>` line, and an optional
    `--result-file` with the outcome as JSON.
  - Other apps find the program with `Store::contactcraft_exe()` in the `contactcraft-store`
    crate. See the README's "Command line for other apps".
- The guides and the MessageCraft AI instructions now explain how to use the editor window.
- The documentation says plainly that ContactCraft never connects to Google or any contacts
  service: contacts come in and go out only through files.

Installed copies of ContactCraft 1.0.0 offer this update the next time they start.

### Download

- **ContactCraft-windows-x64.msi**: the installer (recommended). It installs into Program Files,
  adds a Start Menu shortcut, and offers to launch ContactCraft when it finishes. It replaces an
  earlier version in place.
- **ContactCraft-windows.zip**: the portable version. Unzip it anywhere and run `ContactCraft.exe`.

Your contacts live in `%APPDATA%\ContactCraft`, not in the program's folder, so installing,
updating and uninstalling never touch them.

### Everything else in ContactCraft

- Every field Google Contacts, Outlook, Apple Contacts and vCard can hold, plus your own fields.
- A picture and a theme for each contact (colours, banner, font, text size, background, message
  bubbles, sounds), with a live preview.
- Shared with Craft apps whether ContactCraft is open or not.
- Import from vCard, Google CSV and Outlook CSV files; export to vCard, Google CSV or JSON.
- Clean Up: proper-case names, standard phone and address formats, repeated details removed, and
  duplicates merged, with undo.
- Checks for new versions on start (File › Options to turn it off).

ContactCraft isn't code-signed yet, so Windows SmartScreen may warn the first time you run the
installer. Choose **More info › Run anyway**.
