import hashlib
import importlib.util
import json
import tempfile
import types
import unittest
from pathlib import Path
from unittest.mock import Mock, patch

spec = importlib.util.spec_from_file_location("voice_publisher", Path(__file__).with_name("publish-voice-runtime.py"))
publisher = importlib.util.module_from_spec(spec)
spec.loader.exec_module(publisher)


class VoicePublicationTest(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.directory = Path(self.temporary.name)
        (self.directory / "assets").mkdir()
        (self.directory / "downloads").mkdir()
        libraries = []
        for name in sorted(publisher.NAMES):
            data = name.encode()
            digest = hashlib.sha256(data).hexdigest()
            (self.directory / "downloads" / name).write_bytes(data)
            libraries.append({"name": name, "bytes": len(data), "sha256": digest})
        (self.directory / "assets/voice-runtime.json").write_text(json.dumps({"format": 1, "abi": "arm64-v8a", "libraries": libraries}))
        self.remote = {item[1]: {"Sha256": item[2]["sha256"], "Size": item[2]["bytes"], "Revision": "a" * 40}
                       for item in publisher.expected_files(self.directory)}

    def test_existing_verified_files_skip_auth_and_upload(self):
        sdk = types.ModuleType("modelscope_hub")
        sdk.HubApi = Mock()
        with patch.dict("sys.modules", {"modelscope_hub": sdk}), patch.object(publisher, "remote_files", return_value=self.remote):
            publisher.publish(self.directory)
        sdk.HubApi.assert_not_called()

    def test_no_write_auth_stops_apk_publication(self):
        sdk = types.ModuleType("modelscope_hub")
        sdk.HubApi = Mock(side_effect=Exception("not authenticated"))
        items = publisher.expected_files(self.directory)
        with patch.dict("sys.modules", {"modelscope_hub": sdk}), patch.object(publisher, "remote_files", return_value={}):
            with self.assertRaisesRegex(RuntimeError, "MODELSCOPE_API_TOKEN"):
                publisher.publish(self.directory)

    def test_uploads_only_missing_component_and_checks_result(self):
        sdk = types.ModuleType("modelscope_hub")
        api = Mock()
        sdk.HubApi = Mock(return_value=api)
        item = publisher.expected_files(self.directory)[0]
        missing = dict(self.remote)
        missing.pop(item[1])
        with patch.dict("sys.modules", {"modelscope_hub": sdk}), patch.object(publisher, "remote_files", side_effect=[missing, self.remote]):
            publisher.publish(self.directory)
        api.upload_file.assert_called_once_with(repo_id="bestyize/xiaoban-asr", repo_type="model", path_or_fileobj=item[0],
                                               path_in_repo=item[1], commit_message="Publish Android runtime libraries", disable_tqdm=True)

    def test_corrupt_component_is_rejected_before_remote_access(self):
        item = publisher.expected_files(self.directory)[0]
        item[0].write_bytes(b"invalid")
        with patch.object(publisher, "remote_files") as remote:
            with self.assertRaisesRegex(RuntimeError, "checksum mismatch"):
                publisher.publish(self.directory)
            remote.assert_not_called()

    def test_existing_files_pin_revisions_without_renaming_them(self):
        with patch.object(publisher, "remote_files", return_value=self.remote):
            publisher.publish(self.directory)
        pins = json.loads((self.directory / "published-revisions.json").read_text())
        self.assertEqual(set(pins), publisher.NAMES)
        self.assertTrue(all(pin["revision"] == "a" * 40 for pin in pins.values()))
        self.assertEqual(set(self.remote), publisher.NAMES)

    def test_missing_revision_stops_publication(self):
        self.remote[next(iter(self.remote))]["Revision"] = "master"
        with patch.object(publisher, "remote_files", return_value=self.remote):
            with self.assertRaisesRegex(RuntimeError, "immutable"):
                publisher.publish(self.directory)
        self.assertFalse((self.directory / "published-revisions.json").exists())

    def test_generator_keeps_plain_filenames_and_pins_urls(self):
        generator_spec = importlib.util.spec_from_file_location("runtime_generator", Path(__file__).with_name("prepare-voice-runtime.py"))
        generator = importlib.util.module_from_spec(generator_spec)
        generator_spec.loader.exec_module(generator)
        native = self.directory / "native"
        native.mkdir()
        for name in publisher.NAMES:
            (native / name).write_bytes(name.encode())
        with patch.object(publisher, "remote_files", return_value=self.remote):
            publisher.publish(self.directory)
        generator.publish_files(native, self.directory)
        manifest = json.loads((self.directory / "assets/voice-runtime.json").read_text())
        self.assertEqual({p.name for p in (self.directory / "downloads").glob("*.so")}, publisher.NAMES)
        for lib in manifest["libraries"]:
            self.assertTrue(lib["sources"][0].endswith("FilePath=" + lib["name"]))
            self.assertIn("Revision=" + "a" * 40, lib["sources"][0])
            self.assertNotIn("voice", lib["sources"][0])

    def test_stale_pin_is_not_used_for_changed_library(self):
        generator_spec = importlib.util.spec_from_file_location("runtime_generator", Path(__file__).with_name("prepare-voice-runtime.py"))
        generator = importlib.util.module_from_spec(generator_spec)
        generator_spec.loader.exec_module(generator)
        native = self.directory / "native"
        native.mkdir()
        for name in publisher.NAMES:
            (native / name).write_bytes(name.encode())
        pins = {name: {"sha256": "0" * 64, "revision": "a" * 40} for name in publisher.NAMES}
        (self.directory / "published-revisions.json").write_text(json.dumps(pins))
        generator.publish_files(native, self.directory)
        manifest = json.loads((self.directory / "assets/voice-runtime.json").read_text())
        self.assertTrue(all("Revision=master" in lib["sources"][0] for lib in manifest["libraries"]))


if __name__ == "__main__":
    unittest.main()
