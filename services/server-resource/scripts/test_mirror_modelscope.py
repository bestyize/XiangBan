import hashlib
import os
import tempfile
import unittest

from mirror_modelscope import mirror_url, repo_path, sha256_file


class Pure(unittest.TestCase):
    def test_paths_are_pinned_per_version(self):
        self.assertEqual(repo_path("effect", "aurora", 3), "effect/aurora/v3.zip")
        self.assertNotEqual(repo_path("effect", "aurora", 3), repo_path("effect", "aurora", 4))
        self.assertEqual(
            mirror_url("xiaoban/community-packs", "effect/aurora/v3.zip"),
            "https://modelscope.cn/models/xiaoban/community-packs/resolve/master/effect/aurora/v3.zip",
        )

    def test_sha256_file(self):
        with tempfile.NamedTemporaryFile(delete=False) as f:
            f.write(b"hello" * 300_000)
        try:
            self.assertEqual(sha256_file(f.name), hashlib.sha256(b"hello" * 300_000).hexdigest())
        finally:
            os.unlink(f.name)


if __name__ == "__main__":
    unittest.main()
