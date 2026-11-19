# ContactSolutions 1.3.0 migration and recovery

1.3.0 is unreleased. No installer or live-data migration is authorized by building it.
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

Move the existing Git checkout to `C:/Projects/ContactSolutions` only from an
external directory after the coordinating terminal handoff. Never move a terminal's
live cwd or create a replacement repository. Update Workspace path facts after the
move; keep the same Card ID/routes. Rename the existing GitHub repository LAST,
then update origin and verify its repository ID/history/tags/releases. Do not
publish a release or reuse the old GitHub repository name as part of this cutover.
