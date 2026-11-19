"""Offline regression and adversarial fixtures; no user data or publication."""
import copy
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import zipfile

import release_pipeline as pipeline


class PromotionTests(unittest.TestCase):
    def setUp(self):
        self.commit = "a" * 40
        self.tag = "v" + pipeline.contract()["version"]
        repository = {"id": pipeline.REPOSITORY_ID, "full_name": pipeline.REPOSITORY}
        self.run = {"id": 123, "run_attempt": 2, "repository": repository.copy(),
                    "head_repository": repository.copy(), "head_sha": self.commit,
                    "head_branch": "main", "event": "push", "path": pipeline.WORKFLOW,
                    "workflow_id": pipeline.WORKFLOW_ID, "status": "completed",
                    "conclusion": "success", "html_url": "https://example.invalid/run/123"}
        self.payloads = {name: (name + " fixture").encode() for name in pipeline.ASSETS}
        self.manifest = {"schema": 1, "build": pipeline.contract(), "producer": {
            "repository": pipeline.REPOSITORY, "repository_id": pipeline.REPOSITORY_ID,
            "workflow": pipeline.WORKFLOW, "run_id": 123, "run_attempt": 2,
            "commit": self.commit, "event": "push", "ref": "refs/heads/main"},
            "assets": {name: {"bytes": len(data), "sha256": pipeline.sha256(data)}
                       for name, data in self.payloads.items()}}
        self.artifact = {"id": 456, "name": pipeline.artifact_name(self.commit, 2),
                         "expired": False, "workflow_run": {"id": 123,
                         "repository_id": pipeline.REPOSITORY_ID,
                         "head_repository_id": pipeline.REPOSITORY_ID,
                         "head_branch": "main", "head_sha": self.commit}}

    def archive(self, extras=None):
        output = io.BytesIO()
        with zipfile.ZipFile(output, "w") as bundle:
            bundle.writestr(pipeline.MANIFEST, json.dumps(self.manifest))
            for name, data in self.payloads.items():
                bundle.writestr(name, data)
            for name, data in (extras or {}).items():
                bundle.writestr(name, data)
        data = output.getvalue()
        self.artifact["digest"] = "sha256:" + pipeline.sha256(data)
        return data

    def test_valid_archive_promotes_identical_bytes(self):
        manifest, data = pipeline.unpack_verified(self.archive(), self.artifact, self.run, self.tag)
        self.assertEqual(manifest, self.manifest)
        self.assertEqual(data, self.payloads)

    def test_rejects_untrusted_or_incomplete_producers(self):
        for key, value in [("head_sha", "b" * 40), ("head_branch", "topic"),
                           ("event", "pull_request"), ("event", "workflow_dispatch"),
                           ("path", "other.yml"), ("workflow_id", 9),
                           ("status", "in_progress"), ("conclusion", "failure"),
                           ("conclusion", "cancelled"), ("run_attempt", 0)]:
            with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                pipeline.validate_run({**self.run, key: value}, self.commit)
        for field in ("repository", "head_repository"):
            for key, value in [("id", 9), ("full_name", "other/repository")]:
                run = copy.deepcopy(self.run)
                run[field][key] = value
                with self.subTest(field=field, key=key), self.assertRaises(ValueError):
                    pipeline.validate_run(run, self.commit)

    def test_rejects_wrong_artifact_origin_attempt_digest_expiry(self):
        self.archive()
        for key, value in [("expired", True), ("expired", None), ("name", "old-name"),
                           ("digest", None), ("digest", "sha256:invalid"), ("id", 0)]:
            with self.subTest(key=key), self.assertRaises(ValueError):
                pipeline.validate_artifact({**self.artifact, key: value}, self.run)
        for key in self.artifact["workflow_run"]:
            artifact = copy.deepcopy(self.artifact)
            artifact["workflow_run"][key] = "wrong"
            with self.subTest(key=key), self.assertRaises(ValueError):
                pipeline.validate_artifact(artifact, self.run)

    def test_rejects_missing_or_changed_contract_and_provenance(self):
        for section in ("producer", "build", "assets"):
            for key in self.manifest[section]:
                manifest = copy.deepcopy(self.manifest)
                manifest[section].pop(key)
                with self.subTest(section=section, key=key), self.assertRaises(ValueError):
                    pipeline.validate_manifest(manifest, self.run, self.tag)
        with self.assertRaises(ValueError):
            pipeline.validate_manifest(self.manifest, self.run, "v0.0.0")

    def test_rejects_archive_and_final_byte_tampering(self):
        archive = self.archive()
        with self.assertRaisesRegex(ValueError, "Archive digest"):
            pipeline.unpack_verified(archive + b"tamper", self.artifact, self.run, self.tag)
        self.payloads[pipeline.ASSETS[0]] = b"tamper"
        with self.assertRaisesRegex(ValueError, "Final-byte"):
            pipeline.unpack_verified(self.archive(), self.artifact, self.run, self.tag)

    def test_rejects_missing_extra_and_unsafe_members(self):
        for extra in ("../outside", "unexpected.exe", "/absolute", "subdir/asset"):
            with self.subTest(extra=extra), self.assertRaisesRegex(ValueError, "archive entries"):
                pipeline.unpack_verified(self.archive({extra: b"bad"}), self.artifact, self.run, self.tag)
        self.payloads.pop(pipeline.ASSETS[0])
        with self.assertRaises(ValueError):
            pipeline.unpack_verified(self.archive(), self.artifact, self.run, self.tag)

    def mock_api(self, path, binary=False):
        if path == "actions/runs/123":
            return self.run
        if path == "actions/artifacts/456":
            return self.artifact
        if path == "actions/artifacts/456/zip":
            return self.archive_bytes
        raise AssertionError(f"Unexpected API call: {path}")

    def mock_pages(self, path, key):
        return [self.run] if key == "workflow_runs" else [self.artifact]

    def test_end_to_end_read_only_promotion(self):
        self.archive_bytes = self.archive()
        with tempfile.TemporaryDirectory() as tmp, patch.object(pipeline, "api", self.mock_api), \
                patch.object(pipeline, "pages", self.mock_pages), \
                patch.object(pipeline.subprocess, "check_output", return_value=self.commit + "\n"):
            destination = Path(tmp) / "promoted"
            pipeline.promote(self.commit, self.tag, destination)
            for name, data in self.payloads.items():
                self.assertEqual((destination / name).read_bytes(), data)
            self.assertFalse(json.loads((destination / "promotion-receipt.json").read_text())["publication"])
            with self.assertRaises(FileExistsError):
                pipeline.promote(self.commit, self.tag, destination)

    def test_no_run_or_missing_ambiguous_artifact_fails_closed(self):
        self.archive_bytes = self.archive()
        for runs, artifacts in [([], []), ([self.run], []), ([self.run], [self.artifact] * 2)]:
            with tempfile.TemporaryDirectory() as tmp, patch.object(pipeline, "api", self.mock_api), \
                    patch.object(pipeline, "pages", side_effect=lambda path, key:
                                 runs if key == "workflow_runs" else artifacts), \
                    patch.object(pipeline.subprocess, "check_output", return_value=self.commit), \
                    self.assertRaises(ValueError):
                destination = Path(tmp) / "promoted"
                pipeline.promote(self.commit, self.tag, destination)
            self.assertFalse(destination.exists())

    def test_rerun_during_download_is_rejected(self):
        self.archive_bytes = self.archive()
        calls = []
        def changing_api(path, binary=False):
            if path == "actions/runs/123":
                calls.append(path)
                if len(calls) == 2:
                    return {**self.run, "run_attempt": 3}
            return self.mock_api(path, binary)
        with tempfile.TemporaryDirectory() as tmp, patch.object(pipeline, "api", changing_api), \
                patch.object(pipeline, "pages", self.mock_pages), \
                patch.object(pipeline.subprocess, "check_output", return_value=self.commit), \
                self.assertRaisesRegex(ValueError, "rerun changed"):
            pipeline.promote(self.commit, self.tag, Path(tmp) / "promoted")

    def test_failed_newer_run_is_not_hidden(self):
        newer = {**self.run, "id": 124, "conclusion": "failure"}
        with tempfile.TemporaryDirectory() as tmp, \
                patch.object(pipeline, "pages", return_value=[self.run, newer]), \
                patch.object(pipeline, "api", return_value=newer), \
                patch.object(pipeline.subprocess, "check_output", return_value=self.commit), \
                self.assertRaisesRegex(ValueError, "conclusion"):
            pipeline.promote(self.commit, self.tag, Path(tmp) / "promoted")

    def test_published_bytes_are_downloaded_and_compared(self):
        assets = [{"name": name, "id": index, "size": len(data),
                   "digest": "sha256:" + pipeline.sha256(data)}
                  for index, (name, data) in enumerate(self.payloads.items())]
        release = {"tag_name": self.tag, "draft": False, "prerelease": False, "assets": assets}
        def published_api(path, binary=False):
            if path.startswith("releases/tags/"):
                return release
            return self.payloads[assets[int(path.rsplit("/", 1)[1])]["name"]]
        with tempfile.TemporaryDirectory() as tmp, patch.object(pipeline, "api", published_api):
            directory = Path(tmp)
            (directory / pipeline.MANIFEST).write_text(json.dumps(self.manifest))
            pipeline.verify_published(self.tag, directory)
            self.payloads[pipeline.ASSETS[0]] = b"corrupt downloaded asset"
            with self.assertRaisesRegex(ValueError, "Final-byte"):
                pipeline.verify_published(self.tag, directory)
            release["draft"] = True
            with self.assertRaisesRegex(ValueError, "normal published"):
                pipeline.verify_published(self.tag, directory)

    def test_source_hash_is_checkout_eol_independent(self):
        with tempfile.TemporaryDirectory() as tmp, patch.object(pipeline, "ROOT", Path(tmp)):
            file = Path(tmp) / "source"
            file.write_bytes(b"one\ntwo\n")
            expected = pipeline.source_hash("source")
            file.write_bytes(b"one\r\ntwo\r\n")
            self.assertEqual(expected, pipeline.source_hash("source"))

    def test_tag_must_resolve_to_producer_commit(self):
        with patch.object(pipeline.subprocess, "check_output", return_value=self.commit):
            pipeline.validate_tag(self.commit, self.tag)
            with self.assertRaisesRegex(ValueError, "Tag does not point"):
                pipeline.validate_tag("b" * 40, self.tag)
        with self.assertRaisesRegex(ValueError, "Invalid release tag"):
            pipeline.validate_tag(self.commit, "--bad")

    def test_download_headers_follow_endpoint_contract(self):
        with patch.object(pipeline.subprocess, "run") as run:
            run.return_value.stdout = b"binary fixture"
            self.assertEqual(pipeline.api("actions/artifacts/456/zip", binary=True), b"binary fixture")
            self.assertNotIn("Accept: application/octet-stream", run.call_args.args[0])
            self.assertEqual(pipeline.api("releases/assets/456", binary=True), b"binary fixture")
            self.assertIn("Accept: application/octet-stream", run.call_args.args[0])

    def test_workflow_contract_and_no_duplicate_release_build(self):
        workflow = (pipeline.ROOT / pipeline.WORKFLOW).read_text(encoding="utf-8-sig")
        self.assertIn('toolchain: "' + pipeline.contract()["toolchain"] + '"', workflow)
        self.assertIn('rustup default ' + pipeline.contract()["toolchain"], workflow)
        self.assertIn(pipeline.contract()["toolchain"] + '-x86_64-pc-windows-msvc', workflow)
        self.assertLess(workflow.index('rustup toolchain uninstall'), workflow.index('uses: Swatinem/rust-cache'))
        self.assertEqual(workflow.count("run: ./build.ps1"), 1)
        release = workflow.split("\n  release:\n")[1]
        self.assertNotIn("./build.ps1", release)
        self.assertNotIn("cargo ", release)
        self.assertIn("gh release create", release)
        self.assertNotIn("--clobber", release)
        build = (pipeline.ROOT / "build.ps1").read_text()
        self.assertNotIn('rustup toolchain uninstall', build)
        self.assertIn("cargo test --workspace --locked --quiet", build)
        self.assertIn("cargo build --release --locked", build)


if __name__ == "__main__":
    unittest.main()
