"""Reject private working material and incomplete licensing in source distributions."""

from pathlib import Path, PurePosixPath
import json
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
PRIVATE_PARTS = {
    ".aws", ".claude", ".codex", ".doghouse", ".git", ".solidstate", ".ssh",
    "node_modules", "target", "dist", "__pycache__",
}
PRIVATE_ROOTS = {"local", "artifacts", "build", "data", "evidence"}
PRIVATE_SUFFIXES = {
    ".bin", ".dll", ".dtb", ".dylib", ".elf", ".exe", ".gz", ".hex",
    ".img", ".jks", ".key", ".ko", ".log", ".out", ".p12", ".pem",
    ".pfx", ".pyc", ".pyo", ".rom", ".so", ".tar", ".zip",
}


def main() -> int:
    entries = subprocess.check_output(
        ["git", "ls-files", "--stage", "-z"], cwd=ROOT
    ).decode().split("\0")
    errors = []
    for entry in filter(None, entries):
        metadata, name = entry.split("\t", 1)
        mode, _, stage = metadata.split()
        path = PurePosixPath(name)
        if mode not in {"100644", "100755"} or stage != "0":
            errors.append(f"Not a regular, resolved source file: {name}")
        if (
            set(path.parts) & PRIVATE_PARTS
            or path.parts[0] in PRIVATE_ROOTS
            or any(part.endswith(".app") for part in path.parts)
            or path.name.startswith(".env")
            or path.suffix.lower() in PRIVATE_SUFFIXES
            or (path.suffix == ".dag" and path.parts[0] == "specs")
        ):
            errors.append(f"Private or generated material is tracked: {name}")

    for name in ["LICENSE", "CONTRIBUTING.md", "SECURITY.md", "CODE_OF_CONDUCT.md"]:
        if not (ROOT / name).is_file():
            errors.append(f"Required publication document missing: {name}")

    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"], cwd=ROOT
    ))
    for package in metadata["packages"]:
        if package["id"] not in metadata["workspace_members"]:
            continue
        directory = Path(package["manifest_path"]).parent
        member = directory.relative_to(ROOT)
        if package["license"] != "MIT":
            errors.append(f"Package must declare the project's MIT license: {member}")
        if not (directory / "LICENSE").is_file():
            errors.append(f"Packaged license missing: {member}")
        elif (directory / "LICENSE").read_bytes() != (ROOT / "LICENSE").read_bytes():
            errors.append(f"Packaged license differs from the project license: {member}")
    if json.loads((ROOT / "package.json").read_text()).get("license") != "MIT":
        errors.append("Project tooling package must declare the MIT license")

    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print("Source distribution boundaries and package licenses checked.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
