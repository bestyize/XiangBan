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
            (self.directory / "downloads" / f"android-voice-arm64-{digest}-{name}").write_bytes(data)
            libraries.append({"name": name, "bytes": len(data), "sha256": digest})
        (self.directory / "assets/voice-runtime.json").write_text(json.dumps({"format": 1, "abi": "arm64-v8a", "libraries": libraries}))

    def test_existing_verified_files_skip_auth_and_upload(self):
        sdk = types.ModuleType("modelscope_hub")
        sdk.HubApi = Mock()
        with patch.dict("sys.modules", {"modelscope_hub": sdk}), patch.object(publisher, "missing_files", return_value=[]):
            publisher.publish(self.directory)
        sdk.HubApi.assert_not_called()

    def test_no_write_auth_stops_apk_publication(self):
        sdk = types.ModuleType("modelscope_hub")
        sdk.HubApi = Mock(side_effect=Exception("not authenticated"))
        items = publisher.expected_files(self.directory)
        with patch.dict("sys.modules", {"modelscope_hub": sdk}), patch.object(publisher, "missing_files", return_value=items):
            with self.assertRaisesRegex(RuntimeError, "MODELSCOPE_API_TOKEN"):
                publisher.publish(self.directory)

    def test_uploads_only_missing_component_and_checks_result(self):
        sdk = types.ModuleType("modelscope_hub")
        api = Mock()
        sdk.HubApi = Mock(return_value=api)
        item = publisher.expected_files(self.directory)[0]
        with patch.dict("sys.modules", {"modelscope_hub": sdk}), patch.object(publisher, "missing_files", side_effect=[[item], []]):
            publisher.publish(self.directory)
        api.upload_file.assert_called_once_with(repo_id="bestyize/xiaoban-asr", repo_type="model", path_or_fileobj=item[0],
                                               path_in_repo=item[1], commit_message="Publish pinned Android speech runtime", disable_tqdm=True)

    def test_corrupt_component_is_rejected_before_remote_access(self):
        item = publisher.expected_files(self.directory)[0]
        item[0].write_bytes(b"invalid")
        with patch.object(publisher, "missing_files") as remote:
            with self.assertRaisesRegex(RuntimeError, "checksum mismatch"):
                publisher.publish(self.directory)
            remote.assert_not_called()


if __name__ == "__main__":
    unittest.main()
