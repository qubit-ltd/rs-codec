"""Regression tests for Markdown example extraction and manifest validation."""

import importlib.util
import pathlib
import sys
import unittest

SCRIPT = pathlib.Path(__file__).resolve().parents[1] / "check-doc-examples.py"
SPEC = importlib.util.spec_from_file_location("check_doc_examples", SCRIPT)
CHECKER = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = CHECKER
SPEC.loader.exec_module(CHECKER)


class ExtractFencesTests(unittest.TestCase):
    def test_source_edit_changes_extracted_code(self):
        before = CHECKER.extract_fences("# Topic\n\n```rust\nassert_eq!(1, 1);\n```\n", "guide.md")
        after = CHECKER.extract_fences("# Topic\n\n```rust\nassert_eq!(2, 2);\n```\n", "guide.md")
        self.assertNotEqual(before[0].code, after[0].code)
        self.assertEqual("assert_eq!(2, 2);\n", after[0].code)

    def test_unclosed_rust_fence_is_rejected_with_location(self):
        with self.assertRaisesRegex(CHECKER.CheckError, r"guide.md:3.*unclosed"):
            CHECKER.extract_fences("# Topic\n\n```rust\nlet value = 1;\n", "guide.md")


class ManifestTests(unittest.TestCase):
    def setUp(self):
        self.docs = {
            "en.md": "# Demo\n\n```rust\nassert_eq!(1, 1);\n```\n",
            "zh.md": "# 示例\n\n```rust\nassert_eq!(1, 1);\n```\n",
        }
        self.manifest = {
            "version": 1,
            "pairs": [{"left": "en-case", "right": "zh-case"}],
            "cases": [
                self.case("en-case", "en.md", "Demo"),
                self.case("zh-case", "zh.md", "示例"),
            ],
            "exclusions": [],
        }

    @staticmethod
    def case(case_id, source, heading):
        return {
            "id": case_id,
            "source": source,
            "heading": heading,
            "fence": 1,
            "features": [],
            "prerequisites": [],
            "mode": "run",
            "pair": "zh-case" if case_id == "en-case" else "en-case",
        }

    def assert_invalid(self, manifest, docs, pattern):
        with self.assertRaisesRegex(CHECKER.CheckError, pattern):
            CHECKER.validate_manifest(manifest, docs)

    def test_missing_fence_fails(self):
        self.docs["en.md"] = "# Demo\n"
        self.assert_invalid(self.manifest, self.docs, r"en-case.*missing fence")

    def test_duplicate_location_fails(self):
        self.manifest["cases"].append(self.case("duplicate", "en.md", "Demo"))
        self.assert_invalid(self.manifest, self.docs, r"duplicate location")

    def test_unknown_case_reference_fails(self):
        self.manifest["pairs"][0]["right"] = "absent"
        self.assert_invalid(self.manifest, self.docs, r"unknown case")

    def test_unregistered_fence_fails(self):
        self.docs["en.md"] += "\n```rust\nlet another = 2;\n```\n"
        self.assert_invalid(self.manifest, self.docs, r"unregistered.*en.md")

    def test_english_chinese_code_drift_fails(self):
        self.docs["zh.md"] = self.docs["zh.md"].replace("assert_eq!(1, 1)", "assert_eq!(1, 2)")
        self.assert_invalid(self.manifest, self.docs, r"drift")

    def test_unknown_configuration_field_fails(self):
        self.manifest["cases"][0]["surprise"] = True
        self.assert_invalid(self.manifest, self.docs, r"unknown.*field")


if __name__ == "__main__":
    unittest.main()
