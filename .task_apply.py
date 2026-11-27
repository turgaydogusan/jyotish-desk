#!/usr/bin/env python3
"""Apply a task section from the bhava-sudarshana plan."""
import re
import subprocess
import sys
from pathlib import Path

PLAN = Path("docs/superpowers/plans/2026-11-27-plan-25-bhava-sudarshana.md")
ROOT = Path(".").resolve()


def task_chunk(n: int) -> str:
    text = PLAN.read_text()
    start = text.index(f"### Task {n}:")
    end = text.index(f"### Task {n + 1}:") if n < 8 else len(text)
    return text[start:end]


def apply_new_files(chunk: str) -> list[str]:
    pattern = re.compile(
        r"\*\*Yeni dosya:\*\* `([^`]+)`(?: \(.*?\))?\s*````(?:rust|tsx|ts|md)\n(.*?)````",
        re.S,
    )
    by_path: dict[str, str] = {}
    for path, body in pattern.findall(chunk):
        body = body.rstrip() + "\n"
        by_path[path] = body
    written = []
    for path, body in by_path.items():
        p = ROOT / path
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(body)
        written.append(path)
    return written


def apply_diffs(chunk: str) -> int:
    diffs = re.findall(r"````diff\n(.*?)````", chunk, re.S)
    ok = 0
    for i, d in enumerate(diffs):
        patch = ROOT / f"/tmp/task_diff_{i}.patch"
        patch.write_text(d)
        r = subprocess.run(
            ["git", "apply", "--check", str(patch)],
            cwd=ROOT,
            capture_output=True,
            text=True,
        )
        if r.returncode != 0:
            r2 = subprocess.run(
                ["git", "apply", "--3way", str(patch)],
                cwd=ROOT,
                capture_output=True,
                text=True,
            )
            if r2.returncode != 0:
                print(f"diff {i} FAILED:\n{r.stderr[:1500]}\n{r2.stderr[:1500]}")
                sys.exit(1)
        else:
            subprocess.run(["git", "apply", str(patch)], cwd=ROOT, check=True)
        ok += 1
    return ok


def commit_msg(n: int) -> str:
    text = PLAN.read_text()
    m = re.search(rf"### Task {n}:.*?\n\n.*?git commit -m \"([^\"]+)\"", text, re.S)
    if not m:
        sys.exit(f"No commit message for task {n}")
    return m.group(1)


def main() -> None:
    n = int(sys.argv[1])
    chunk = task_chunk(n)
    ndiffs = apply_diffs(chunk)
    files = apply_new_files(chunk)
    print(f"Task {n}: {len(files)} new files, {ndiffs} diffs")


if __name__ == "__main__":
    main()