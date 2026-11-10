"""Checks that every commit since the commit convention started carries a
`Produced-by:` trailer (goal G19, decision D74).

The convention started with commit 0d5045d. Merge commits are not checked.
Run from the repository root:
    python tools/check_trailers.py
Standard library only. Needs the full git history (fetch-depth: 0 in CI).
"""
from __future__ import annotations

import re
import subprocess
import sys

FIRST = "0d5045d2c4be51d98b31603ef409134c2ff75593"  # first commit under the D74 convention
TRAILER = re.compile(r"^Produced-by: \S")


def git(*args: str) -> str:
    return subprocess.run(["git", *args], check=True, capture_output=True,
                          text=True, encoding="utf-8").stdout


def main() -> int:
    # Merge commits are skipped: GitHub's pull-request merge commit carries no trailer.
    log = git("log", "--no-merges", "--format=%H%x1f%s%x1f%B%x1e", f"{FIRST}^..HEAD")
    commits = [c.strip("\n") for c in log.split("\x1e") if c.strip()]
    bad = []
    for c in commits:
        sha, subject, body = c.split("\x1f", 2)
        if not any(TRAILER.match(line) for line in body.splitlines()):
            bad.append((sha[:7], subject))
    for sha, subject in bad:
        print(f"FAIL  {sha}  no Produced-by trailer: {subject}")
    print(f"{len(commits) - len(bad)} of {len(commits)} commits since {FIRST[:7]} carry a Produced-by trailer")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
