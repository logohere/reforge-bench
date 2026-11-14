"""Shared build identity and resource layout for native desktop packages."""

from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import os
import shlex
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def build_desktop(debug, agent_debug, target):
    bun = shutil.which("bun")
    if not bun:
        raise SystemExit("Bun 1.3.14 must be available on PATH.")
    subprocess.run([bun, "install", "--frozen-lockfile", "--ignore-scripts"], cwd=ROOT / "web", check=True)
    subprocess.run([bun, "run", "build"], cwd=ROOT / "web", check=True)
    command = ["cargo", "build", "--locked", "-p", "reforge-bench-desktop"]
    if target:
        command.extend(["--target", target])
    if not debug:
        command.append("--release")
    if agent_debug:
        command.extend(["--features", "agent-debug"])
    build_env=os.environ.copy()
    if not debug:
        encoded=build_env.get("CARGO_ENCODED_RUSTFLAGS")
        flags=encoded.split("\x1f") if encoded else shlex.split(build_env.get("RUSTFLAGS",""))
        flags.extend([f"--remap-path-prefix={Path.home()}=/user",f"--remap-path-prefix={ROOT}=/reforge"])
        build_env["CARGO_ENCODED_RUSTFLAGS"]="\x1f".join(flags)
    subprocess.run(command, cwd=ROOT, env=build_env, check=True)
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"], cwd=ROOT))
    version = next(package["version"] for package in metadata["packages"] if package["name"] == "reforge-bench-desktop")
    receipt = {"version":version, "agent_debug":agent_debug, "debug":debug,
        "target":target or subprocess.check_output(["rustc", "--print", "host-tuple"], cwd=ROOT).decode().strip(),
        "source_revision":None,"source_dirty":None,"source_sha256":None,
        "built_at":datetime.now(timezone.utc).isoformat()}
    try:
        revision=subprocess.check_output(["git","rev-parse","HEAD"],cwd=ROOT,stderr=subprocess.DEVNULL).decode().strip()
        names=subprocess.check_output(["git","ls-files","--cached","--others","--exclude-standard","-z"],cwd=ROOT).decode().split("\0")
        source=hashlib.sha256()
        for name in sorted(set(filter(None,names))):
            path=ROOT/name
            if path.is_file():
                source.update(name.encode());source.update(path.read_bytes())
        receipt.update(source_revision=revision,source_dirty=bool(subprocess.check_output(["git","status","--porcelain"],cwd=ROOT)),source_sha256=source.hexdigest())
    except subprocess.CalledProcessError:
        pass
    cache = Path(metadata["target_directory"])
    if target:
        cache = cache / target
    binary = cache / ("debug" if debug else "release") / ("reforge-bench-desktop.exe" if "windows" in receipt["target"] else "reforge-bench-desktop")
    return binary, receipt


def copy_resources(resources):
    for name in json.loads((ROOT/"web/resources.json").read_text()):
        relative=Path(name)
        if relative.is_absolute() or ".." in relative.parts:
            raise SystemExit("Invalid packaged resource path")
        source=(ROOT/"web"/relative).resolve()
        if not source.is_file() or ROOT/"web" not in source.parents:
            raise SystemExit("Resource must be a regular file inside web")
        destination=resources/"web"/relative
        destination.parent.mkdir(parents=True,exist_ok=True)
        shutil.copy2(source,destination)
    shutil.copytree(ROOT/"web/dist",resources/"web/dist")
    shutil.copy2(ROOT/"web/build.json",resources/"web/build.json")
    shutil.copy2(ROOT / "LICENSE", resources / "LICENSE")
    shutil.copy2(ROOT / "THIRD_PARTY_NOTICES.md", resources / "THIRD_PARTY_NOTICES.md")
    shutil.copytree(ROOT / "experiments/rune/licenses", resources / "licenses")
