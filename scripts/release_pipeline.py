"""Promote only final bytes from this repository's successful exact-commit main CI.

No build, publish, install or fallback is performed by the promotion command.
GitHub CLI supplies authenticated read-only API requests; tokens are never logged.
"""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import re
import subprocess
import tomllib
from urllib.parse import quote, urlencode
import zipfile

ROOT = Path(__file__).resolve().parents[1]
REPOSITORY = "william-pyle/ContactSolutions"
REPOSITORY_ID = 1456328695
WORKFLOW = ".github/workflows/build.yml"
WORKFLOW_ID = 417034250
TARGET = "x86_64-pc-windows-msvc"
ASSETS = ("ContactSolutions-windows-x64.msi", "ContactSolutions-windows.zip")
MANIFEST = "release-manifest.json"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def read_toml(name):
    return tomllib.loads((ROOT / name).read_text(encoding="utf-8-sig"))


def source_hash(name):
    # Windows checkout converts LF to CRLF; hash normalized source, not checkout EOLs.
    return sha256((ROOT / name).read_bytes().replace(b"\r\n", b"\n"))


def contract():
    return {
        "version": read_toml("Cargo.toml")["package"]["version"],
        "target": TARGET,
        "toolchain": read_toml("rust-toolchain.toml")["toolchain"]["channel"],
        "features": "default",
        "profile": "release",
        "locked": True,
        "wix": "5.0.2",
        "lock_sha256": source_hash("Cargo.lock"),
        "workflow_sha256": source_hash(WORKFLOW),
        "build_sha256": source_hash("build.ps1"),
    }


def api(path, binary=False):
    args = ["gh", "api", f"repos/{REPOSITORY}/{path}"]
    if binary:
        args += ["-H", "Accept: application/octet-stream"]
    result = subprocess.run(args, check=True, stdout=subprocess.PIPE)
    return result.stdout if binary else json.loads(result.stdout)


def pages(path, key):
    # Never silently accept a truncated list.
    for page in range(1, 101):
        separator = "&" if "?" in path else "?"
        response = api(f"{path}{separator}per_page=100&page={page}")
        items = response[key]
        yield from items
        if len(items) < 100:
            return
    raise ValueError("API pagination limit reached")


def validate_run(run, commit):
    require(re.fullmatch(r"[0-9a-f]{40}", commit), "Invalid commit")
    for field in ("repository", "head_repository"):
        require(run[field]["id"] == REPOSITORY_ID and
                run[field]["full_name"] == REPOSITORY, "Wrong producer repository")
    expected = {"head_sha": commit, "head_branch": "main", "event": "push",
                "path": WORKFLOW, "workflow_id": WORKFLOW_ID,
                "status": "completed", "conclusion": "success"}
    for field, value in expected.items():
        require(run.get(field) == value, f"Untrusted producer: {field}")
    require(isinstance(run.get("id"), int) and run["id"] > 0, "Invalid run ID")
    require(isinstance(run.get("run_attempt"), int) and run["run_attempt"] > 0,
            "Invalid run attempt")


def artifact_name(commit, attempt):
    return f"ContactSolutions-windows-{commit}-{attempt}"


def validate_artifact(artifact, run):
    require(artifact.get("name") == artifact_name(run["head_sha"], run["run_attempt"]),
            "Wrong artifact name or attempt")
    require(artifact.get("expired") is False, "Artifact expired")
    require(isinstance(artifact.get("id"), int) and artifact["id"] > 0,
            "Invalid artifact ID")
    require(re.fullmatch(r"sha256:[0-9a-f]{64}", artifact.get("digest") or ""),
            "Missing or invalid immutable artifact digest")
    expected = {"id": run["id"], "repository_id": REPOSITORY_ID,
                "head_repository_id": REPOSITORY_ID, "head_branch": "main",
                "head_sha": run["head_sha"]}
    for field, value in expected.items():
        require(artifact["workflow_run"].get(field) == value,
                f"Artifact provenance mismatch: {field}")


def validate_manifest(manifest, run, tag):
    require(manifest.get("schema") == 1, "Unsupported manifest schema")
    expected = contract()
    require(tag == "v" + expected["version"], "Tag/version mismatch")
    require(manifest.get("build") == expected, "Build contract mismatch")
    producer = {"repository": REPOSITORY, "repository_id": REPOSITORY_ID,
                "workflow": WORKFLOW, "run_id": run["id"],
                "run_attempt": run["run_attempt"], "commit": run["head_sha"],
                "event": "push", "ref": "refs/heads/main"}
    require(manifest.get("producer") == producer, "Manifest producer mismatch")
    require(set(manifest.get("assets", {})) == set(ASSETS), "Wrong asset set")


def verify_payloads(manifest, payloads):
    require(set(payloads) == set(ASSETS), "Missing or unexpected assets")
    for name, data in payloads.items():
        require(len(data) > 0, f"Empty asset: {name}")
        require(manifest["assets"].get(name) ==
                {"bytes": len(data), "sha256": sha256(data)},
                f"Final-byte mismatch: {name}")


def unpack_verified(archive, artifact, run, tag):
    validate_run(run, run["head_sha"])
    validate_artifact(artifact, run)
    require("sha256:" + sha256(archive) == artifact["digest"], "Archive digest mismatch")
    with zipfile.ZipFile(io.BytesIO(archive)) as bundle:
        names = bundle.namelist()
        require(len(names) == 3 and set(names) == {*ASSETS, MANIFEST},
                "Unexpected archive entries (including duplicate or unsafe paths)")
        require(all(i.file_size <= 512 * 1024 * 1024 for i in bundle.infolist()),
                "Artifact member too large")
        manifest = json.loads(bundle.read(MANIFEST))
        validate_manifest(manifest, run, tag)
        payloads = {name: bundle.read(name) for name in ASSETS}
    verify_payloads(manifest, payloads)
    return manifest, payloads


def make_manifest(directory):
    build = contract()
    compiler = subprocess.check_output(["rustc", "-vV"], text=True)
    require(f"release: {build['toolchain']}\n" in compiler, "Wrong Rust compiler")
    require(f"host: {TARGET}\n" in compiler, "Wrong Rust host target")
    # The producer uses native Windows defaults; reject hidden target/features overrides.
    for key in ("CARGO_BUILD_TARGET", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"):
        require(not os.environ.get(key), f"Unexpected build override: {key}")
    require(os.environ["GITHUB_REPOSITORY"] == REPOSITORY, "Wrong repository")
    require(int(os.environ["GITHUB_REPOSITORY_ID"]) == REPOSITORY_ID, "Wrong repository ID")
    require(os.environ["GITHUB_WORKFLOW_REF"].startswith(f"{REPOSITORY}/{WORKFLOW}@"),
            "Wrong workflow")
    payloads = {name: (directory / name).read_bytes() for name in ASSETS}
    manifest = {"schema": 1, "build": build, "producer": {
        "repository": REPOSITORY, "repository_id": REPOSITORY_ID, "workflow": WORKFLOW,
        "run_id": int(os.environ["GITHUB_RUN_ID"]),
        "run_attempt": int(os.environ["GITHUB_RUN_ATTEMPT"]),
        "commit": os.environ["GITHUB_SHA"], "event": os.environ["GITHUB_EVENT_NAME"],
        "ref": os.environ["GITHUB_REF"]}, "assets": {
            name: {"bytes": len(data), "sha256": sha256(data)}
            for name, data in payloads.items()}}
    verify_payloads(manifest, payloads)
    (directory / MANIFEST).write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(manifest, indent=2))


def promote(commit, tag, directory):
    require(re.fullmatch(r"[0-9a-f]{40}", commit), "Invalid commit")
    require(subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip() == commit,
            "Checkout differs from requested producer commit")
    require(tag == "v" + contract()["version"], "Tag/version mismatch")
    query = urlencode({"head_sha": commit, "branch": "main", "event": "push"})
    runs = list(pages(f"actions/workflows/{WORKFLOW_ID}/runs?{query}", "workflow_runs"))
    require(runs, "No exact-commit main CI producer; do not rebuild on the tag")
    # A newer failed/in-progress rerun must not be masked by an older green run.
    run = api(f"actions/runs/{max(runs, key=lambda r: r['id'])['id']}")
    validate_run(run, commit)
    artifacts = [a for a in pages(f"actions/runs/{run['id']}/artifacts", "artifacts")
                 if a["name"] == artifact_name(commit, run["run_attempt"])]
    require(len(artifacts) == 1, "Missing or ambiguous producer artifact")
    artifact = api(f"actions/artifacts/{artifacts[0]['id']}")
    validate_artifact(artifact, run)
    archive = api(f"actions/artifacts/{artifact['id']}/zip", binary=True)
    manifest, payloads = unpack_verified(archive, artifact, run, tag)
    # Recheck the producer after download, detecting a rerun begun during validation.
    latest = api(f"actions/runs/{run['id']}")
    validate_run(latest, commit)
    require(latest["run_attempt"] == run["run_attempt"], "Producer rerun changed")
    directory.mkdir(parents=True, exist_ok=False)
    for name, data in payloads.items():
        (directory / name).write_bytes(data)
    (directory / MANIFEST).write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    receipt = {"run_id": run["id"], "artifact_id": artifact["id"],
               "archive_digest": artifact["digest"], "commit": commit,
               "producer_url": run["html_url"], "publication": False}
    (directory / "promotion-receipt.json").write_text(json.dumps(receipt, indent=2) + "\n",
                                                       encoding="utf-8")
    print(json.dumps(receipt, indent=2))


def verify_published(tag, directory):
    manifest = json.loads((directory / MANIFEST).read_text(encoding="utf-8"))
    require(tag == "v" + manifest["build"]["version"], "Published tag/version mismatch")
    release = api(f"releases/tags/{quote(tag, safe='')}")
    require(release["tag_name"] == tag and not release["draft"] and not release["prerelease"],
            "Not a normal published release")
    assets = release["assets"]
    require(len(assets) == len(ASSETS) and {a["name"] for a in assets} == set(ASSETS),
            "Published asset set differs")
    payloads = {}
    for asset in assets:
        expected = manifest["assets"][asset["name"]]
        require(asset["size"] == expected["bytes"] and
                asset.get("digest") == "sha256:" + expected["sha256"],
                "Published asset metadata mismatch")
        payloads[asset["name"]] = api(f"releases/assets/{asset['id']}", binary=True)
    verify_payloads(manifest, payloads)
    print(f"Verified published bytes for {tag}; no installation performed.")


def validate_tag(commit, tag):
    require(re.fullmatch(r"v[0-9]+\.[0-9]+\.[0-9]+", tag), "Invalid release tag")
    resolved = subprocess.check_output(
        ["git", "rev-parse", "--verify", f"refs/tags/{tag}^{{commit}}"], text=True).strip()
    require(resolved == commit, "Tag does not point to producer commit")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["manifest", "promote", "verify-ci", "verify-published"])
    parser.add_argument("--directory", required=True, type=Path)
    parser.add_argument("--sha")
    parser.add_argument("--tag")
    args = parser.parse_args()
    if args.command == "manifest":
        make_manifest(args.directory)
    elif args.command == "promote":
        require(args.sha and args.tag, "--sha and --tag required")
        validate_tag(args.sha, args.tag)
        promote(args.sha, args.tag, args.directory)
    elif args.command == "verify-ci":
        require(args.sha, "--sha required")
        # Same provenance/final-byte gate, but no tag is created or claimed to exist.
        promote(args.sha, "v" + contract()["version"], args.directory)
    else:
        require(args.tag, "--tag required")
        verify_published(args.tag, args.directory)


if __name__ == "__main__":
    main()
