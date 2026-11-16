"""Pure ordinary revision comparisons; no process or executable fixtures."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("source_check", Path(__file__).resolve().parents[1] / "check-worker-source.py")
source = importlib.util.module_from_spec(spec)
spec.loader.exec_module(source)


class EquivalentWrapperTest(unittest.TestCase):
    def test_same_library_tree_is_accepted(self):
        source.require_same_library("755f32c45a4c30e805e3c7fe1db89dcb4dcc6e41", "755f32c45a4c30e805e3c7fe1db89dcb4dcc6e41")

    def test_an_ordinary_library_revision_needs_its_own_source_review(self):
        with self.assertRaisesRegex(SystemExit, "changed the pinned library tree"):
            source.require_same_library("1" * 40, "2" * 40)


if __name__ == "__main__":
    unittest.main()
