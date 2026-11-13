"""Archive a reviewed commit, never the local working directory or Git history."""

import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def main() -> None:
    if subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT):
        raise SystemExit("Commit reviewed source changes before creating a distribution.")
    subprocess.run([sys.executable, "scripts/check-distribution.py"], cwd=ROOT, check=True)
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"], cwd=ROOT
    ))
    versions = {p["version"] for p in metadata["packages"] if p["id"] in metadata["workspace_members"]}
    if len(versions) != 1:
        raise SystemExit("Source release requires one shared workspace version.")
    version = versions.pop()
    if not re.fullmatch(r"\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?", version):
        raise SystemExit("Invalid source distribution version.")
    commit = subprocess.check_output(["git", "rev-parse", "--short=12", "HEAD"], cwd=ROOT).decode().strip()
    name = f"reforge-bench-{version}-{commit}"
    directory = ROOT / "dist"
    if directory.is_symlink():
        raise SystemExit("Distribution directory must not be a symlink.")
    directory.mkdir(exist_ok=True)
    output = directory / f"{name}.tar.gz"
    with tempfile.NamedTemporaryFile(dir=directory, suffix=".tar.gz", delete=False) as file:
        temporary = Path(file.name)
    try:
        subprocess.run(
            ["git", "archive", "--format=tar.gz", f"--prefix={name}/", f"--output={temporary}", "HEAD"],
            cwd=ROOT,
            check=True,
        )
        with tarfile.open(temporary) as archive:
            members = archive.getmembers()
            if f"{name}/LICENSE" not in {member.name for member in members}:
                raise SystemExit("Distribution is missing its license.")
            if any(member.issym() or member.islnk() for member in members):
                raise SystemExit("Source distribution contains a link.")
        os.replace(temporary, output)
    finally:
        temporary.unlink(missing_ok=True)
    digest = hashlib.sha256(output.read_bytes()).hexdigest()
    print(f"dist/{output.name}\nSHA-256 {digest}")


if __name__ == "__main__":
    main()
