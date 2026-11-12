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
    def test_non_rust_block_hides_nested_rust(self):
        doc = "# Real\n````markdown\n# Fake\n```rust\nnot rust\n```\n````\n"
        self.assertEqual([], CHECKER.extract_fences(doc, "guide.md"))

    def test_non_rust_heading_does_not_change_following_rust(self):
        doc = "# Real\n```text\n# Fake\n```\n```rust\nassert!(true);\n```\n"
        fences = CHECKER.extract_fences(doc, "guide.md")
        self.assertEqual(1, len(fences))
        self.assertEqual("Real", fences[0].heading)
        self.assertEqual(1, fences[0].fence)
        self.assertEqual(5, fences[0].line)

    def test_fence_boundaries_preserve_code_and_location(self):
        cases = [
            ("~~~rust\n", "tilde\n", "~~~\n", "tilde\n"),
            ("````rust\n", "```\n", "```\n````\n", "```\n```\n"),
            ("```rust\n", "~~~\n", "~~~\n```\n", "~~~\n~~~\n"),
            ("```rust\n", "", "````\n", ""),
            ("   ```rust\n", "indented\n", "   ```\n", "indented\n"),
        ]
        for opener, code, closer, expected_code in cases:
            with self.subTest(opener=opener, closer=closer):
                doc = "# Topic\n" + opener + code + closer
                fences = CHECKER.extract_fences(doc, "guide.md")
                self.assertEqual(1, len(fences))
                self.assertEqual(("guide.md", "Topic", 1), fences[0].location)
                self.assertEqual(2, fences[0].line)
                self.assertEqual(expected_code, fences[0].code)

    def test_crlf_fence_preserves_code_and_location(self):
        doc = "# Topic\r\n```rust\r\nassert!(true);\r\n```\r\n"
        fences = CHECKER.extract_fences(doc, "guide.md")
        self.assertEqual(1, len(fences))
        self.assertEqual(("guide.md", "Topic", 1), fences[0].location)
        self.assertEqual(2, fences[0].line)
        self.assertEqual("assert!(true);\r\n", fences[0].code)

    def test_opening_and_closing_fences_allow_zero_to_three_spaces(self):
        for indent in range(4):
            with self.subTest(indent=indent):
                spaces = " " * indent
                doc = f"# Topic\n{spaces}```rust\nassert!(true);\n{spaces}```\n"
                fences = CHECKER.extract_fences(doc, "guide.md")
                self.assertEqual(1, len(fences))
                self.assertEqual(("guide.md", "Topic", 1), fences[0].location)
                self.assertEqual(2, fences[0].line)
                self.assertEqual("assert!(true);\n", fences[0].code)

    def test_unclosed_non_rust_block_hides_remainder(self):
        doc = "# Topic\n```text\n# Fake\n```rust\nnot rust\n"
        self.assertEqual([], CHECKER.extract_fences(doc, "guide.md"))

    def test_unclosed_rust_fence_is_rejected_with_opener_line(self):
        with self.assertRaisesRegex(CHECKER.CheckError, r"guide.md:2: unclosed Rust fence"):
            CHECKER.extract_fences("# Topic\n  ```rust\nassert!(true);\n", "guide.md")

    def test_backtick_in_info_does_not_open_fence(self):
        doc = "# Topic\n```rust`extra\nordinary text\n```rust\nassert!(true);\n```\n"
        fences = CHECKER.extract_fences(doc, "guide.md")
        self.assertEqual(1, len(fences))
        self.assertEqual(("guide.md", "Topic", 1), fences[0].location)
        self.assertEqual(4, fences[0].line)
        self.assertEqual("assert!(true);\n", fences[0].code)

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
