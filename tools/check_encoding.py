"""Checks that the tables of docs/encoding.md are consistent (spec 004, criterion 2).

- Event vocabulary: id ranges are contiguous from 0 and end at 287 (288 ids).
- Actions: slot ranges are contiguous from 0 and end at 159 (160 slots).
- State token: offsets are contiguous from 0, each block's rows sum to the
  block's stated width, and the total is 1,382.

Run from the repository root:  python tools/check_encoding.py
Standard library only.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

DOC = Path(__file__).resolve().parent.parent / "docs" / "encoding.md"
EXPECTED = {"events": 288, "actions": 160, "state": 1382}
DASH = "–"  # en dash used in the ranges


def section(text: str, heading_start: str) -> str:
    """The text of the section whose heading starts with `heading_start`."""
    m = re.search(rf"^## {re.escape(heading_start)}.*?$", text, flags=re.M)
    if not m:
        raise SystemExit(f"FAIL  section '{heading_start}' not found")
    rest = text[m.end():]
    nxt = re.search(r"^## ", rest, flags=re.M)
    return rest[: nxt.start()] if nxt else rest


def cells(line: str) -> list[str]:
    return [c.strip() for c in line.strip().strip("|").split("|")]


def parse_range(s: str) -> tuple[int, int]:
    s = s.replace("*", "")
    if DASH in s:
        a, b = s.split(DASH)
        return int(a), int(b)
    return int(s), int(s)


def check_ranges(name: str, body: str, first_header: str) -> tuple[bool, str]:
    """Rows are `| range | count | ...`. Returns (ok, message)."""
    rows = [cells(l) for l in body.splitlines() if l.startswith("|")]
    rows = [r for r in rows if r and re.match(r"^\d", r[0])]
    if not rows:
        return False, f"{name}: no rows found under '{first_header}'"
    nxt = 0
    for r in rows:
        lo, hi = parse_range(r[0])
        count = int(r[1])
        if lo != nxt:
            return False, f"{name}: range {r[0]} starts at {lo}, expected {nxt}"
        if hi - lo + 1 != count:
            return False, f"{name}: range {r[0]} has {hi - lo + 1} ids but the count says {count}"
        nxt = hi + 1
    want = EXPECTED[name]
    if nxt != want:
        return False, f"{name}: total {nxt}, expected {want}"
    return True, f"{name}: {len(rows)} rows, contiguous, total {nxt}"


def check_state(body: str) -> tuple[bool, str]:
    rows = [cells(l) for l in body.splitlines() if l.startswith("|")]
    nxt, block, block_width, block_sum, blocks = 0, None, 0, 0, 0
    total_row = None
    for r in rows:
        if len(r) < 3 or r[0] in ("Block", "") or set(r[0]) <= {"-"}:
            continue
        if r[0].startswith("**Total"):
            total_row = int(r[1].strip("*").replace(",", ""))
            continue
        if r[0].startswith("**"):  # block header: | **A. Cards** | **702** | | |
            if block is not None and block_sum != block_width:
                return False, f"state: block {block} rows sum to {block_sum}, header says {block_width}"
            block, block_width, block_sum = r[0].strip("*"), int(r[1].strip("*")), 0
            blocks += 1
            continue
        width = int(r[1])
        lo, hi = parse_range(r[2])
        if lo != nxt:
            return False, f"state: '{r[0]}' starts at {lo}, expected {nxt}"
        if hi - lo + 1 != width:
            return False, f"state: '{r[0]}' spans {hi - lo + 1} but the width says {width}"
        nxt = hi + 1
        block_sum += width
    if block is not None and block_sum != block_width:
        return False, f"state: block {block} rows sum to {block_sum}, header says {block_width}"
    if nxt != EXPECTED["state"]:
        return False, f"state: total {nxt}, expected {EXPECTED['state']}"
    if total_row != nxt:
        return False, f"state: the Total row says {total_row}, the rows sum to {nxt}"
    return True, f"state: {blocks} blocks, contiguous, total {nxt}"


def main() -> int:
    text = DOC.read_text(encoding="utf-8")
    results = [
        check_ranges("events", section(text, "3. Tokens"), "Id"),
        check_ranges("actions", section(text, "5. Actions"), "Slot"),
        check_state(section(text, "4. State token")),
    ]
    for ok, msg in results:
        # ASCII only, so the output is safe on any Windows console code page.
        msg = msg.replace(DASH, "-").encode("ascii", "replace").decode("ascii")
        print(("PASS  " if ok else "FAIL  ") + msg)
    failed = sum(1 for ok, _ in results if not ok)
    print(f"\n{len(results) - failed} of {len(results)} checks passed")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
