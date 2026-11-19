# Release workflow

## Sequence and authorization

Finish source, version, documentation and workflow changes before tagging. For a product
release update the three versions in Cargo.toml, store/Cargo.toml and package.json, plus
packaging/release-notes.md. Workflow-only changes keep the existing product version.

Push the reviewed commit to main and wait for the entire **Build** run for that exact SHA
to finish successfully, including artifact upload and post-job steps. The `windows` job
remains the application CI gate. A local pass, older commit, PR merge commit, manual run,
cache hit or uploaded artifact alone does not satisfy this gate.

With explicit authorization for a new release, tag the successful commit with `v<version>`
and push that tag separately. Do not push main and a release tag together. Do not recreate
v1.3.0 or alter its assets. GitHub release creation fails if the release already exists.
There is no release overwrite/rebuild fallback.

Publication is separate from installation, restart and live-data migration. No installer
is executed by this workflow. Existing unsigned status, updater MSI/ZIP names, MSI upgrade
identity and data migration behavior are unchanged. Future signing must happen before
manifest creation; verification must cover the signed final bytes.

## Producer and promotion trust

There is one workflow, `.github/workflows/build.yml` (Build, ID 417034250), in repository
`william-pyle/ContactSolutions` (ID 1456328695). Its triggers are main pushes, `v*` tag pushes,
pull requests and manual dispatch. No site deployment is present.

Non-tag runs execute `windows`: pipeline regression checks, Rust setup, cache restore,
WiX setup, `build.ps1` (rename audit, locked workspace tests, locked release compilation,
ZIP and MSI packaging), manifest generation and immutable artifact upload. The Rust
toolchain is pinned to 1.100.0, the compiler observed in the successful 1.3.0 release.
Native target is x86_64-pc-windows-msvc, default features, release profile, WiX 5.0.2.

Tag pushes execute only `release`. They do not depend on the skipped `windows` job in
the tag run. Instead, the promotion script requires completed, successful exact-SHA main
push CI from the expected repository, head repository and workflow. The newest matching
run must pass; an older green run cannot mask a newer failed/in-progress attempt. The tag
must resolve to the checked-out producer commit and match the source version.

The producer artifact name includes the commit and run attempt. Its manifest binds
repository, workflow path, run ID/attempt, commit, version, target, toolchain, features,
profile, locked dependencies, WiX version and source hashes. Source hashes normalize CRLF
to LF so Windows and Linux checkouts agree. Each final MSI/ZIP has an exact size and SHA-256.
The consumer also validates GitHub's artifact ID, provenance and archive SHA-256, rejects
expiry/missing metadata, duplicate/extra/path-traversing archive entries and changed bytes,
then rechecks the producer attempt after download. There is no cross-commit reuse.

Only MSI, ZIP and manifest are uploaded, with compression disabled because the packages
are already compressed. Artifact retention is 30 days. If an artifact expires or predates
this manifest format, rerun the exact main **push** run under approved release preparation,
wait for success and use that attempt. Do not rebuild on the tag or accept a different SHA.

The release job has contents-write and actions-read permissions; ordinary CI has
contents-read. Checkout does not persist credentials. PR artifacts/manual artifacts cannot
pass the release gate. No signing, environment approval or security scan existed in the
previous workflow; none was removed. Main branch protection was reported absent by GitHub
during the inventory; repository settings were not changed.

After creating a release, the job checks the asset set, API sizes/digests and independently
downloads each published asset to compare final bytes. A failed post-publication check
means publication occurred but verification failed: investigate; do not automatically
replace assets or rerun creation. Retain the producer artifact and run evidence.

## Avoiding repeated work

The cache action keeps its compiler/host, Cargo.lock/manifests and Rust environment
invalidation. It also hashes every installed compiler: observed main run 44454776158
missed because the hosted image retained unused Rust 1.99.1 alongside pinned 1.100.0,
whereas run 44454294420 had only the 1.100.0 compiler version. Before caching, CI now
selects the pinned default and removes unused toolchains on its disposable hosted runner.
Local toolchains are never removed by build.ps1. This stabilizes the inventory without
dropping compiler/target/environment hashes or weakening dependency invalidation.
Only main push runs save caches; PRs/manual runs may restore. Caches are
compilation accelerators, never trusted release artifacts. `--locked` prevents an
unreviewed dependency resolution. The existing cache missed after the repository rename,
then hit on the tag; no evidence justified adding another large cache or weakening keys.
WiX setup took 11–12 seconds, so additional installer caching is deferred.

Superseded nonrelease runs cancel within event/ref (or PR number). Separate event groups
prevent a diagnostic dispatch from cancelling required main push CI. Tag runs use their
own ref group with cancellation disabled. Different PRs remain parallel. Tests and
compilation remain sequential within a producer to share Cargo outputs and avoid lock
contention; there is no duplicate packaging matrix. PR/main runs are not interchangeable:
PRs validate merge results, main supplies exact-commit release evidence.

Run focused checks after workflow changes:

```powershell
python -m unittest discover -s scripts -p test_release_pipeline.py -v
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check-rename.ps1
actionlint .github/workflows/build.yml
git diff --check
```

After successful main CI, validate real artifact retrieval without tagging or publishing
(use a new output directory and the actual checked-out SHA):

```powershell
python scripts/release_pipeline.py verify-ci --sha <full-commit-sha> --directory .tools/verified-ci
```

Do not rerun already-green full suites without a changed-input reason. Workflow/build
changes justify one new exact-commit CI producer run. No local application rebuild is
needed merely to repeat that same check. Shared rules should distinguish build evidence,
publication, installation and observed application behavior; corrections to shared policy
belong to Alpha's assigned global scope, not this repository.

## Measured baseline (2026-11-19 UTC)

At commit `17245278eef33d2be5fd57f227dadcd6b941bcf3`:

| Run | Job duration | Build/test/package | Cache behavior |
| --- | ---: | ---: | --- |
| [Main CI 44435401434](https://github.com/william-pyle/ContactSolutions/actions/runs/44435401434) | 585 s | 503 s | Miss; save 36 s |
| [v1.3.0 tag 44436802310](https://github.com/william-pyle/ContactSolutions/actions/runs/44436802310) | 245 s | 164 s | Hit; restore 23 s |

The duplicate tag build/setup is removed by design. The former 245-second tag job is a
measured baseline, not a measured saving: promotion still incurs runner/API/download time.
No optimized tag publication is exercised without authorization for a new release.

References: [artifact metadata/digests](https://docs.github.com/en/rest/actions/artifacts),
[concurrency](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-workflow-concurrency),
[Rust cache keys and save policy](https://github.com/Swatinem/rust-cache).
