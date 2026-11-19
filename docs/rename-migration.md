# ContactSolutions 1.3.0 migration and recovery

This guide covers version 1.3.0. Building or publishing a package does not install
it or migrate live data; migration occurs only when the application is started.
The renamed application is the first product in a wider rename; older MailCraft and
MessageCraft versions are not supported against its new paths/interfaces. Their
source changes and deployments are separate work.

## First start with existing data

The new default store and profile are `%APPDATA%/ContactSolutions`. UI settings are
`contactsolutions-ui.json` in that profile, even when `CONTACTSOLUTIONS_DATA` points
the shared store elsewhere. Contact/group IDs, schema version, content, photos,
themes, fonts and cleanup backups keep their meaning.

When `%APPDATA%/ContactCraft` exists, startup attempts an offline copy to a sibling
`ContactSolutions.migration-staging` directory. The app never closes another app.
Every source file is held against access/write/deletion during the Windows copy;
an open or inaccessible file causes a visible startup error. Close the consumers
yourself only in an authorized maintenance window, then retry.

The copy includes SQLite WAL files. Only the staged database is opened for WAL
recovery and an integrity check. Each copied file is checked byte-for-byte before
database recovery. Settings JSON must be valid and is copied under its new name;
unknown settings fields survive subsequent settings saves. Old executable discovery
metadata is removed from the staged database; the new app records its new executable.
No contact/group records or schema are rewritten by this rename.

Only a validated staging directory is renamed into the destination. The old folder
is retained unchanged as a recovery copy, not used as the active store. A
`migration-receipt.json` records source path and SHA-256 file hashes. Subsequent
startup refuses a changed retained source instead of silently choosing or merging
divergent stores. Keep the receipt with the new data. Do not resume old consumers.

For a legacy custom store, keep `CONTACTCRAFT_DATA` as the migration-source setting
and set `CONTACTSOLUTIONS_DATA` to a separate new destination. The old variable is
only a migration reader, not an ongoing alias. After checking the new store and
retaining the old backup, remove the old environment setting. The new variable
alone selects the new active store. Never point both at the same or nested folders.

## Conflicts and interrupted attempts

- Existing old and new folders without a matching receipt: stop; preserve both.
  Do not overwrite either or automatically merge their contacts.
- Both UI-settings filenames in the old folder: resolve which is authoritative
  after backing up both; the app does not guess.
- Invalid settings/database or inaccessible files: startup reports the error and
  does not activate the new store. Correct the original only from a verified backup.
- A leftover `.migration-staging` or `.migration-lock` blocks retry. Confirm no
  migration process is running; preserve the failed stage separately. Then move
  the stale lock/stage out of the way and retry. Do not blindly delete them.
- A `.json.pending` settings file indicates an interrupted save or a collision.
  Preserve and compare it with the active settings before resolving it.

Migration requires enough free space for the complete copy. Any copy/validation
failure leaves the original untouched and retains an incomplete stage for review.
The operation is offline on Windows; do not use it to copy a live shared store on
another platform. Reparse points and symbolic links are refused.

## Rollback

Stop only processes you are authorized to stop. Preserve the new folder, receipt
and any post-migration writes before reverting binaries or environment settings.
The retained old folder is the pre-migration state, not a backup of later edits.
Restoring that state alone would lose later edits; export/reconcile them first.
Neither installer rollback nor renaming the source checkout rolls contact data back.
Do not install/uninstall a product merely to test this procedure on an active host.

## Export/import is not a complete backup or identity-preserving migration

The v1.2.4 "ContactCraft (everything)" JSON export with **all contacts** retains
contact details, custom/unknown fields and groups in the file. It does not embed
photo, theme-image or font files. JSON import clears the photo reference and Google
link metadata; theme file references alone do not restore those external files.
Exporting only shown contacts excludes groups. App settings are not exported.

The old vCard export includes custom/unknown contact data through its JSON extension
and can embed contact photos. It excludes groups and settings, and does not bundle
theme-image/font files. Imported photos larger than 512 pixels may be resized.
CSV is less complete and is not the preferred recovery format.

Final import creates new contact and group IDs. JSON group members are remapped to
the imported or matched contact IDs. "Skip contacts I already have" is enabled by
default: it matches email, phone or name and skips the record, without merging its
fields into the destination. Groups are not deduplicated. Importing the same file
twice can create duplicate groups; disabling contact skipping can duplicate contacts.

The old installer does not remove the user data folder. If it remains, the renamed
app attempts guarded folder migration on startup before an import, so importing
again is usually unnecessary. For full recovery, keep a consistent complete-folder
backup (including any custom store and the separate settings profile). JSON plus
vCard are useful additional backups, but importing both is not a guaranteed merge.
Validate copies in an isolated profile: counts, group membership, custom fields,
pictures/themes/settings and identities should be checked before discarding originals.

`python scripts/check-legacy-export.py` exercises actual v1.2.4 export code and the
current reader/import/save/duplicate functions using synthetic data. It does not
uninstall/install anything or read the owner's profile. This is separate from the
native pointer/keyboard and installer validation still needed before deployment.

## Intentionally retained old names

- Migration source folder/environment/settings names and old executable metadata key.
- `X-CONTACTCRAFT-JSON` and old JSON exports are read to preserve existing exports;
  new exports use ContactSolutions names. No old-version writers/aliases are retained.
- Historical release notes, tags, releases/assets, Git commits and dated evidence.
- Alpha project ID `contactcraft`, routes `#/contactcraft/0` and `#/contactcraft/1`,
  and canonical `projects/contactcraft/` required by Alpha's ID-based lookup.
- MSI UpgradeCode `9CC84440-9F4C-4EBB-BCCC-468A2E210F25` is the same upgrade family.
  Changed installation components have new GUIDs and paths.

## Installer and source cutover

The MSI installs `ContactSolutions.exe` under Program Files/ContactSolutions with
new shortcut and resource names. Installing/uninstalling does not migrate user data;
that occurs only when the app is run. Interactive and unattended upgrade testing
must be done in a disposable Windows environment before deployment approval.

The owner selected `C:/Projects/ContactSolutions` as the authoritative checkout
after verifying matching commits, trees and tags in both directories. Retain the
old `C:/Projects/ContactCraft` directory untouched; this explicit decision supersedes
the earlier move-only plan. Never move a terminal's live cwd or close its session.
Workspace facts point to the selected checkout while keeping the same Card ID/routes.
The existing GitHub repository was renamed to `william-pyle/ContactSolutions` after
cutover checks. Repository identity, history and historical tags/releases are
preserved. Release publication is a separately authorized operation; it does not
authorize installation or live-data migration. Do not modify the retained checkout
or reuse the old GitHub repository name.
