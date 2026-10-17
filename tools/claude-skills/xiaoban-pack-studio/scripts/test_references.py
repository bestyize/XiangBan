"""references/ holds copies of the desktop docs so the skill still works when copied out of the repo.
This checks the copies have not drifted; it is skipped when the skill runs outside the repository."""
import unittest
from pathlib import Path

SKILL = Path(__file__).resolve().parents[1]
DOCS = SKILL.parents[2] / "client" / "xiaoban-desktop-wallpaper" / "docs"
FILES = ["resource-pack-format.md", "effects.md", "effect-pack-example.json"]
EXAMPLE = ["manifest.json", "index.html", "game.js"]


@unittest.skipUnless(DOCS.is_dir(), "not inside the repository")
class ReferencesInSync(unittest.TestCase):
    def test_copies_match_the_desktop_docs(self):
        for name in FILES:
            self.assertEqual(
                (SKILL / "references" / name).read_bytes(),
                (DOCS / name).read_bytes(),
                f"{name} differs from client/xiaoban-desktop-wallpaper/docs; copy it over again",
            )

    def test_the_desktop_game_example_matches(self):
        for name in EXAMPLE:
            self.assertEqual(
                (SKILL / "references" / "examples" / "desk-pal" / name).read_bytes(),
                (DOCS / "examples" / "desk-pal" / name).read_bytes(),
                f"examples/desk-pal/{name} differs; copy it over again",
            )


if __name__ == "__main__":
    unittest.main()
