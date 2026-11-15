## ContactCraft 1.2.0

Contacts and groups on Windows, with pictures and themes shared with other applications.

### New in 1.2.0

- **Contact groups.** Create named groups with a standard theme and any mix of saved contacts and
  phone numbers that do not have a ContactCraft contact. Add and remove members in the group pane.
  Other applications can create and update groups through `contactcraft-store`. Groups are included
  in ContactCraft JSON exports. Existing databases gain the group table automatically.
- **Banner pictures.** Choose a picture for a contact or group banner, alongside the existing
  colour and pattern choices.
- **Delete confirmation.** Pressing Enter now confirms the contact deletion dialog.
- **Theme editor wording.** The editor uses “App Default” and no longer names other applications.
- **Updated icon.** The blue tile now has a solid white person silhouette.
- **Group navigation for other applications.** `--open-group <id>` opens an existing group in the
  full app. `--new-group --group-name <name> [--group-number <number> ...]` creates and opens one.

Installed copies of ContactCraft 1.1.0 offer this update the next time they start.

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
