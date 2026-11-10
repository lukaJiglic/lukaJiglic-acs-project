"""Checks the acceptance criteria of docs/specs/001-repo-skeleton.md.

Prints one line per criterion and exits non-zero if any fails.
Run from the repository root:  python tools/check_skeleton.py
Standard library only.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
results: list[tuple[str, bool, str]] = []


def read(rel: str) -> str | None:
    p = ROOT / rel
    return p.read_text(encoding="utf-8") if p.is_file() else None


def check(label: str, ok: bool, detail: str = "") -> None:
    results.append((label, ok, detail))


def has_all(text: str | None, needles: list[str]) -> tuple[bool, str]:
    if text is None:
        return False, "file missing"
    missing = [n for n in needles if n.lower() not in text.lower()]
    return (not missing), ("missing: " + ", ".join(missing) if missing else "")


def goal_ids(text: str) -> list[str]:
    # Goal lines look like "- G4. ..." or "- G18 (T1). ..."
    return re.findall(r"^- (G\d+)(?: \(T\d\))?\. ", text, flags=re.M)


# 1. README.md
readme = read("README.md")
ok, d = has_all(readme, [
    "## What this is", "## Goals", "## Agentic stack", "## Install",
    "## Run", "## Demo", "## Not in scope", "## Licences", "docs/planning",
    "claude-fable-5-1", "claude-fable-5-2", "not yet available",
])
check("1  README.md has the required sections", ok, d)

plan = read("docs/planning/plan.md")
if readme is None or plan is None:
    check("1b README goals match plan.md section 4", False, "file missing")
else:
    section = plan.split("## 4. Measurement", 1)[-1].split("## 5.", 1)[0]
    want, got = goal_ids(section), goal_ids(readme)
    expected = [f"G{i}" for i in range(1, 20)]
    ok = want == expected and got == expected
    check("1b README goals match plan.md section 4", ok,
          "" if ok else f"plan has {len(want)} goals, README has {len(got)}")

# 2. CLAUDE.md
ok, d = has_all(read("CLAUDE.md"), [
    "Session routine", "Verification before each commit", "Commit convention",
    "Produced-by:", "Ask, do not assume", "cmd", "other tarok engines",
    "No LLM narration", "docs/planning/README.md", "docs/progress.md",
])
check("2  CLAUDE.md has the standing instructions", ok, d)

# 3. docs/progress.md
ok, d = has_all(read("docs/progress.md"), [
    "Entry format", "2026-11-06", "2026-11-09", "2026-11-10",
    "Done", "Measurements", "Broken", "Next",
])
check("3  docs/progress.md has the format and past sessions", ok, d)

# 4. docs/architecture.md
ok, d = has_all(read("docs/architecture.md"), [
    "planned, not built", "tarok-engine", "WASM", "PyO3", "src/app", "src/train",
])
check("4  docs/architecture.md describes the planned architecture", ok, d)

# 5. docs/specs/README.md
ok, d = has_all(read("docs/specs/README.md"), [
    "Status", "Goal", "Acceptance criteria", "Checks", "Files expected to change",
    "Out of scope", "Tool and model", "Results", "NNN",
])
check("5  docs/specs/README.md has the spec template", ok, d)

# 6. Licences
ok1, d1 = has_all(read("LICENSE"), ["MIT License", "2026", "Luka Jelenko Iglic"])
ok2, d2 = has_all(read("LICENSE-assets"), ["CC BY 4.0", "2026", "Luka Jelenko Iglic",
                                            "creativecommons.org/licenses/by/4.0"])
check("6  LICENSE (MIT) and LICENSE-assets (CC BY 4.0)", ok1 and ok2, "; ".join(x for x in (d1, d2) if x))

# 7. Layout
dirs = ["src/engine", "src/app", "src/train", "src/assets/cards", "models"]
missing = [x for x in dirs if read(x + "/README.md") is None]
wf = (ROOT / ".github" / "workflows").is_dir()
check("7  directory layout with README files", not missing and wf,
      ("missing README in: " + ", ".join(missing)) if missing else ("" if wf else ".github/workflows missing"))

# 8. CI workflow file (the run itself is checked on GitHub)
ok, d = has_all(read(".github/workflows/ci.yml"), ["push", "main", "check_trailers.py", "fetch-depth: 0"])
check("8  ci.yml runs the trailer check on push to main", ok, d)

# 9. Line endings
ga = read(".gitattributes")
ok = ga is not None and "eol=lf" in ga
check("9  .gitattributes forces LF", ok, "" if ok else "eol=lf not found")

width = max(len(r[0]) for r in results)
failed = 0
for label, ok, detail in results:
    print(f"{'PASS' if ok else 'FAIL'}  {label.ljust(width)}  {detail}")
    failed += 0 if ok else 1
print(f"\n{len(results) - failed} of {len(results)} checks passed (they cover criteria 1-9; criterion 10 and the CI run are checked by hand)")
sys.exit(1 if failed else 0)
