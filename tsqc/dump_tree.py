#!/usr/bin/env python
"""
Create a single-file snapshot of the project tree without duplicate paths.

Examples
--------
# dump only the real source tree
python tools/dump_tree.py -o snapshot.txt -i src

# default (whole project, but unique relative paths)
python tools/dump_tree.py
"""
from pathlib import Path
import mimetypes, argparse, textwrap, sys

# ─────────────────────────────────────────────────────────────────────────────
EXCLUDE_DIRS = {
    ".git", ".venv", "target", ".github", "dist", "build",
    "__pycache__", ".idea", ".vscode", "_old", "backup", ".history",
    ".ipynb_checkpoints", ".txt",
}
ALLOWED_EXTS = {".py", ".rs", ".toml", ".json", ".md", ".txt", ".yml", ".yaml"}
MAX_BYTES_DEFAULT  = 100_000   # skip huge generated files
MAX_LINES_DEFAULT  = 4_000
# ─────────────────────────────────────────────────────────────────────────────

def excluded(path: Path) -> bool:
    return any(part in EXCLUDE_DIRS for part in path.parts)

def looks_text(path: Path) -> bool:
    mime, _ = mimetypes.guess_type(path)
    return mime is None or mime.startswith("text/") or mime.endswith("+json")

def dump_project(out: Path, root: Path,
                 include_glob: str | None,
                 max_bytes: int, max_lines: int):
    seen: set[str] = set()
    dumped   = 0
    skipped  = 0

    with out.open("w", encoding="utf-8") as fh:
        files = sorted(root.rglob("*"))
        if include_glob:
            files = [p for p in files if p.match(include_glob)]

        for p in files:
            if p.is_dir() or excluded(p):
                continue
            if p.suffix.lower() not in ALLOWED_EXTS:
                continue
            rel = str(p.relative_to(root))
            if rel in seen:
                skipped += 1
                continue
            seen.add(rel)

            if not looks_text(p) or p.stat().st_size > max_bytes:
                continue

            fh.write(f"{'='*5} {rel} {'='*5}\n")
            try:
                lines = p.read_text(encoding="utf-8", errors="replace").splitlines()
            except Exception as e:
                fh.write(f"[skip: {e}]\n\n")
                continue

            if len(lines) > max_lines:
                lines = lines[:max_lines] + [f"... ({len(lines)-max_lines} lines truncated)"]
            fh.write("\n".join(lines) + "\n\n")
            dumped += 1

    print(f"Snapshot written to {out}  ({dumped} files, {skipped} duplicates skipped)")

# ─────────────────────────────────────────────────────────────────────────────
def main(argv: list[str] | None = None):
    ap = argparse.ArgumentParser(
        formatter_class=argparse.RawTextHelpFormatter,
        description=textwrap.dedent("""\
            Dump all text source files into a single txt while ignoring duplicates.

            Typical usage:
              python tools/dump_tree.py -i src          # only real code
              python tools/dump_tree.py -o snapshot.txt # whole repo
        """))
    ap.add_argument("-o", "--output", default="project_snapshot.txt",
                    help="Output file (default: %(default)s)")
    ap.add_argument("-i", "--include", metavar="GLOB", default=None,
                    help="Only include paths matching this glob (e.g. 'src/**')")
    ap.add_argument("-n", "--lines", type=int, default=MAX_LINES_DEFAULT,
                    help="Max lines per file (default: %(default)s)")
    ap.add_argument("-b", "--bytes", type=int, default=MAX_BYTES_DEFAULT,
                    help="Max file size in bytes (default: %(default)s)")
    args = ap.parse_args(argv)

    dump_project(Path(args.output), Path.cwd(),
                 args.include, args.bytes, args.lines)

if __name__ == "__main__":
    main(sys.argv[1:])
