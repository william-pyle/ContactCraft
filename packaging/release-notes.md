## ContactCraft 1.0.0

Your contacts on Windows, with every detail, a picture and a theme for each person, shared with
MessageCraft, MailCraft and every Craft app.

### Download

- **ContactCraft-windows-x64.msi**: the installer (recommended). It installs into Program Files,
  adds a Start Menu shortcut, and offers to launch ContactCraft when it finishes.
- **ContactCraft-windows.zip**: the portable version. Unzip it anywhere and run `ContactCraft.exe`.

Your contacts live in `%APPDATA%\ContactCraft`, not in the program's folder, so installing,
updating and uninstalling never touch them.

### What's in it

- **Every detail about a person**: every field Google Contacts, Outlook, Apple Contacts and vCard
  can hold, as many phones, emails, addresses, websites and dates as you need, with any label, and
  your own fields for anything else.
- **A picture and a theme** for each contact: banner colours and pattern, font, text size and
  colour, background colour or picture, message bubble colours, emoji and sounds, with a live
  preview.
- **Shared with Craft apps** whether ContactCraft is open or not: MessageCraft and MailCraft can
  read and change the same contacts, pictures and themes.
- **Import** from Google Contacts (vCard or Google CSV), Outlook (CSV) and iPhone (vCard);
  **export** to vCard, Google CSV or ContactCraft's own file.
- **Clean Up**: names in proper case without stray symbols, phone numbers and addresses in one
  standard format, repeated emails and numbers removed, and duplicates found and merged (you choose
  which entry wins). A backup is saved first and the clean-up can be undone.
- **Updates**: ContactCraft checks for a new release when it starts and offers to download and
  install it (you can turn this off in File › Options).
- The Craft family's look: ribbon, label pane, list and card; light, dark or follow Windows.

### Getting started

Open ContactCraft from the Start menu and click **Import** to bring in the file you exported from
Google Contacts or Outlook (File › Import shows how), or **New Contact** to start fresh. To look
around with sample contacts first, run `ContactCraft.exe --demo`.

ContactCraft isn't code-signed yet, so Windows SmartScreen may warn the first time you run the
installer. Choose **More info › Run anyway**.

### Privacy

ContactCraft never connects to Google (or any contacts service): your contacts stay on your computer. To bring contacts over from another service, export them there as a vCard or CSV file and import that file.
