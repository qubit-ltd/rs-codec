#!/usr/bin/env python3
"""Compile Rust examples directly from the project's Markdown documentation."""

from __future__ import annotations

import argparse
import json
import pathlib
import re
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[1]
MANIFEST_PATH = ROOT / "scripts" / "doc-examples.json"
DOC_PATHS = ("README.md", "README.zh_CN.md", "doc/user_guide.md", "doc/user_guide.zh_CN.md")


class CheckError(Exception):
    pass


@dataclass(frozen=True)
class Fence:
    source: str
    heading: str
    fence: int
    line: int
    code: str

    @property
    def location(self) -> tuple[str, str, int]:
        return (self.source, self.heading, self.fence)


def _strict_extract(markdown: str, source: str) -> list[Fence]:
    # This implementation preserves exact code and counts fences per nearest heading.
    lines = markdown.splitlines(keepends=True)
    headings: list[tuple[int, str]] = []
    counts: dict[str, int] = {}
    result: list[Fence] = []
    index = 0
    while index < len(lines):
        line = lines[index]
        h = re.match(r"^ {0,3}(#{1,6})[ \t]+(.+?)[ \t]*#*[ \t]*(?:\r?\n)?$", line)
        if h:
            level, title = len(h.group(1)), h.group(2).strip()
            while headings and headings[-1][0] >= level:
                headings.pop()
            headings.append((level, title))
            index += 1
            continue
        fence = re.match(r"^ {0,3}(`{3,}|~{3,})([^\r\n]*)(?:\r?\n)?$", line)
        if not fence:
            index += 1
            continue
        marker, info = fence.group(1), fence.group(2).strip()
        if marker[0] == "`" and "`" in info:
            index += 1
            continue
        is_rust = bool(re.match(r"^(?:rust|rs)(?:\s|,|$)", info, re.I) or re.match(r"^\{\.?rust(?:[ ,}]|$)", info, re.I))
        close_re = re.compile(r"^ {0,3}" + re.escape(marker[0]) + "{" + str(len(marker)) + r",}[ \t]*(?:\r?\n)?$")
        end = index + 1
        while end < len(lines) and not close_re.match(lines[end]):
            end += 1
        if end == len(lines):
            if is_rust:
                raise CheckError(f"{source}:{index + 1}: unclosed Rust fence")
            break
        if is_rust:
            title = headings[-1][1] if headings else ""
            counts[title] = counts.get(title, 0) + 1
            result.append(Fence(source, title, counts[title], index + 1, "".join(lines[index + 1:end])))
        index = end + 1
    return result


# Keep this public name as the tested API.
extract_fences = _strict_extract


def _require_keys(item: dict[str, Any], required: set[str], allowed: set[str], label: str) -> None:
    if not isinstance(item, dict):
        raise CheckError(f"{label}: expected an object")
    missing, unknown = required - item.keys(), item.keys() - allowed
    if missing:
        raise CheckError(f"{label}: missing fields {sorted(missing)}")
    if unknown:
        raise CheckError(f"{label}: unknown fields {sorted(unknown)}")


def validate_manifest(manifest: dict[str, Any], documents: dict[str, str]) -> dict[str, Fence]:
    _require_keys(manifest, {"version", "pairs", "cases", "exclusions"}, {"version", "pairs", "cases", "exclusions"}, "manifest")
    if manifest["version"] != 1:
        raise CheckError("manifest: unsupported version")
    for field in ("pairs", "cases", "exclusions"):
        if not isinstance(manifest[field], list):
            raise CheckError(f"manifest: {field} must be a list")
    if not manifest["cases"]:
        raise CheckError("manifest: at least one executable case is required")
    fences: dict[str, list[Fence]] = {name: extract_fences(text, name) for name, text in documents.items()}
    cases: dict[str, Fence] = {}
    occupied: set[tuple[str, str, int]] = set()
    locations: dict[tuple[str, str, int], str] = {}
    for case in manifest["cases"]:
        _require_keys(case, {"id", "source", "heading", "fence", "features", "prerequisites", "mode", "pair"}, {"id", "source", "heading", "fence", "features", "prerequisites", "mode", "pair", "wrapper"}, "case")
        case_id = case["id"]
        if not isinstance(case_id, str) or not case_id:
            raise CheckError("case: id must be a non-empty string")
        if case_id in cases:
            raise CheckError(f"duplicate case ID {case_id}")
        if not isinstance(case["source"], str) or not isinstance(case["heading"], str):
            raise CheckError(f"{case_id}: source and heading must be strings")
        if not isinstance(case["fence"], int) or isinstance(case["fence"], bool) or case["fence"] < 1:
            raise CheckError(f"{case_id}: fence must be a positive integer")
        if case["pair"] is not None and not isinstance(case["pair"], str):
            raise CheckError(f"{case_id}: pair must be a case ID or null")
        source, location = case["source"], (case["source"], case["heading"], case["fence"])
        if source not in fences:
            raise CheckError(f"{case_id}: unknown source {source}")
        if location in occupied:
            raise CheckError(f"{case_id}: duplicate location {location}")
        match = next((f for f in fences[source] if f.location == location), None)
        if match is None:
            raise CheckError(f"{case_id}: missing fence {source} heading={case['heading']!r} fence={case['fence']}")
        if case.get("wrapper") not in {None, "result_main"}:
            raise CheckError(f"{case_id}: unknown wrapper {case.get('wrapper')!r}")
        if case["mode"] not in {"run", "no_run", "compile_fail"}:
            raise CheckError(f"{case_id}: unknown mode {case['mode']!r}")
        if not isinstance(case["features"], list) or not all(
            isinstance(x, str) and x in {"io", "json", "registry"} for x in case["features"]
        ):
            raise CheckError(f"{case_id}: unknown features")
        if not isinstance(case["prerequisites"], list) or not all(isinstance(x, str) for x in case["prerequisites"]):
            raise CheckError(f"{case_id}: prerequisites must be case IDs")
        occupied.add(location)
        locations[location] = case_id
        cases[case_id] = match
    for exclusion in manifest["exclusions"]:
        _require_keys(exclusion, {"source", "heading", "fence", "reason"}, {"source", "heading", "fence", "reason"}, "exclusion")
        if not isinstance(exclusion["source"], str) or not isinstance(exclusion["heading"], str):
            raise CheckError("exclusion: source and heading must be strings")
        if not isinstance(exclusion["fence"], int) or isinstance(exclusion["fence"], bool) or exclusion["fence"] < 1:
            raise CheckError("exclusion: fence must be a positive integer")
        location = (exclusion["source"], exclusion["heading"], exclusion["fence"])
        if location in occupied:
            raise CheckError(f"duplicate location {location}")
        if not isinstance(exclusion["reason"], str) or not exclusion["reason"].strip():
            raise CheckError(f"exclusion {location}: empty reason")
        if not any(f.location == location for f in fences.get(exclusion["source"], [])):
            raise CheckError(f"exclusion: missing fence {location}")
        occupied.add(location)
    for source, items in fences.items():
        unregistered = [f for f in items if f.location not in occupied]
        if unregistered:
            f = unregistered[0]
            raise CheckError(f"unregistered Rust fence {source}:{f.line} heading={f.heading!r} fence={f.fence}")
    for case in manifest["cases"]:
        for prerequisite in case["prerequisites"]:
            if prerequisite not in cases:
                raise CheckError(f"{case['id']}: unknown prerequisite {prerequisite!r}")
            if prerequisite == case["id"]:
                raise CheckError(f"{case['id']}: self prerequisite")
    paired_ids: set[tuple[str, str]] = set()
    for pair in manifest["pairs"]:
        _require_keys(pair, {"left", "right"}, {"left", "right"}, "pair")
        left, right = pair["left"], pair["right"]
        if not isinstance(left, str) or not isinstance(right, str):
            raise CheckError("pair: left and right must be case IDs")
        if left not in cases or right not in cases:
            raise CheckError(f"unknown case in pair {left!r}, {right!r}")
        if (left, right) in paired_ids or (right, left) in paired_ids:
            raise CheckError(f"duplicate pair {left!r}, {right!r}")
        if cases[left].code.strip() != cases[right].code.strip():
            raise CheckError(f"drift between paired cases {left} and {right}")
        paired_ids.add((left, right))
        if manifest["cases"][[c["id"] for c in manifest["cases"]].index(left)]["pair"] != right:
            raise CheckError(f"{left}: pair field does not match pair declaration")
        if manifest["cases"][[c["id"] for c in manifest["cases"]].index(right)]["pair"] != left:
            raise CheckError(f"{right}: pair field does not match pair declaration")
    for case in manifest["cases"]:
        peer = case["pair"]
        if peer is not None and peer not in cases:
            raise CheckError(f"{case['id']}: unknown paired case {peer!r}")
        if peer is not None and (case["id"], peer) not in paired_ids and (peer, case["id"]) not in paired_ids:
            raise CheckError(f"{case['id']}: pair reference has no pair declaration")
    return cases


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=pathlib.Path, default=MANIFEST_PATH)
    parser.add_argument("--doc-root", type=pathlib.Path, default=ROOT)
    args = parser.parse_args()
    try:
        manifest = json.loads(args.manifest.read_text(encoding="utf-8"))
        doc_root = args.doc_root.resolve()
        documents = {path: (doc_root / path).read_text(encoding="utf-8") for path in DOC_PATHS}
        if set(documents) != set(DOC_PATHS):
            raise CheckError(f"documents: expected exactly {DOC_PATHS}, got {sorted(documents)}")
        cases = validate_manifest(manifest, documents)
        run_cases(cases, manifest)
    except (OSError, json.JSONDecodeError, CheckError, subprocess.CalledProcessError) as error:
        print(error, file=sys.stderr)
        return 1
    return 0


def run_cases(cases: dict[str, Fence], manifest: dict[str, Any]) -> None:
    configured = {case["id"]: case for case in manifest["cases"]}
    groups: dict[tuple[str, ...], list[str]] = {}
    for case_id, case in configured.items():
        groups.setdefault(tuple(sorted(case["features"])), []).append(case_id)
    temporary = pathlib.Path(tempfile.mkdtemp(prefix="qubit-doc-examples-"))
    failed = False
    for group_index, (features, case_ids) in enumerate(sorted(groups.items()), 1):
        group = temporary / f"features-{group_index}"
        feature_label = ",".join(features) or "core"
        print(f"checking {feature_label} group ({len(case_ids)} cases)", flush=True)
        members = [f"cases/{case_id}" for case_id in case_ids]
        group.mkdir(parents=True)
        (group / "Cargo.toml").write_text(
            "[workspace]\nresolver = \"3\"\nmembers = " + json.dumps(members) + "\n",
            encoding="utf-8",
        )
        for case_id in case_ids:
            config, fence = configured[case_id], cases[case_id]
            package = group / "cases" / case_id
            (package / "src").mkdir(parents=True)
            feature_list = ", ".join(json.dumps(x) for x in features)
            dependencies = [f"qubit-codec = {{ path = {json.dumps(str(ROOT))}, features = [{feature_list}] }}"]
            if "io" in features:
                dependencies.append('qubit-io = "^0.19"')
            if "json" in features and ("serde_json" in fence.code):
                dependencies.append('serde_json = "^1.0"')
            (package / "Cargo.toml").write_text(
                "[package]\nname = " + json.dumps("doc-" + case_id.replace("_", "-")) +
                "\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[dependencies]\n" +
                "\n".join(dependencies) + "\n",
                encoding="utf-8",
            )
            parts = []
            for prerequisite_id in config["prerequisites"]:
                prerequisite = cases[prerequisite_id]
                parts.extend("# " + line if line.strip() else "#" for line in prerequisite.code.splitlines())
            code_lines = fence.code.rstrip("\n").splitlines()
            if config.get("wrapper") == "result_main":
                parts.append("# fn main() -> Result<(), Box<dyn std::error::Error>> {")
                parts.extend(code_lines)
                parts.extend(["# Ok(())", "# }"])
            else:
                parts.extend(code_lines)
            mode = {"run": "", "no_run": ",no_run", "compile_fail": ",compile_fail"}[config["mode"]]
            markdown = "```rust" + mode + "\n" + "\n".join(parts) + "\n```\n"
            # JSON string encoding keeps original Markdown text out of shell syntax.
            source = "#![doc = " + json.dumps(markdown, ensure_ascii=False) + "]\n"
            (package / "src" / "lib.rs").write_text(source, encoding="utf-8")
            print(f"{fence.source}:{fence.line}: {case_id} features={','.join(features) or 'core'}", flush=True)
        command = ["cargo", "test", "--doc", "--manifest-path", str(group / "Cargo.toml"), "--workspace"]
        result = subprocess.run(command, cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        if result.returncode:
            failed = True
            print(result.stdout, file=sys.stderr)
            print(f"doc-example group failed; fixture retained at {group}", file=sys.stderr)
        else:
            print(f"{feature_label} group passed", flush=True)
    if failed:
        print(f"generated fixture root retained at {temporary}", file=sys.stderr)
        raise CheckError("one or more documentation example groups failed")
    import shutil
    shutil.rmtree(temporary)


if __name__ == "__main__":
    raise SystemExit(main())
