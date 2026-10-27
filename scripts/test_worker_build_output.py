"""Output admission with ordinary text only; never publish a synthetic executable."""
import os
from pathlib import Path
import tempfile
import unittest
from worker_build_output import validate_output


class WorkerOutput(unittest.TestCase):
    def test_fresh_output_and_existing_manifest_preservation(self):
        root = Path(tempfile.mkdtemp(prefix="worker-output-", dir=os.environ["FEAT164_D2_TEST_ROOT"]))
        self.assertEqual(validate_output(str(root)), root)
        file = root / "current.json"
        file.write_text('{"ordinary":"retained"}\n')
        before = file.read_bytes()
        with self.assertRaisesRegex(ValueError, "already contains"):
            validate_output(str(root))
        self.assertEqual(file.read_bytes(), before)

    def test_relative_output_refused(self):
        with self.assertRaisesRegex(ValueError, "canonical absolute"):
            validate_output("ordinary-relative-directory")


if __name__ == "__main__":
    unittest.main()
