import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("csv_summary", Path(__file__).parents[1] / "scripts/csv_summary.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

class CsvSummaryTests(unittest.TestCase):
    def run_csv(self, content):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "input.csv"
            path.write_text(content, encoding="utf-8")
            return module.summarize(path, "amount")

    def test_exact_decimal_and_blank_values(self):
        result = self.run_csv('\ufeffname,amount\n"a,b",0.1\nc,0.2\nd,\n')
        self.assertEqual(result["sum"], "0.3")
        self.assertEqual(result["records"], 3)
        self.assertEqual(result["blank_values"], 1)
        self.assertEqual(result["coverage"], "all_records")

    def test_large_decimals_are_not_rounded(self):
        result = self.run_csv("amount\n123456789012345678901234567890.1\n0.2\n")
        self.assertEqual(result["sum"], "123456789012345678901234567890.3")

    def test_rejects_invalid_number(self):
        with self.assertRaisesRegex(ValueError, "record 1"):
            self.run_csv("amount\nunknown\n")

    def test_rejects_non_finite_values(self):
        with self.assertRaisesRegex(ValueError, "Non-finite"):
            self.run_csv("amount\nNaN\n")

if __name__ == "__main__":
    unittest.main()

