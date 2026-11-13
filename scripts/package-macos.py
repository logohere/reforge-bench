"""Build the native application with its own resources and macOS identity."""

from pathlib import Path
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
import plistlib
import shutil
import shlex
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--debug", action="store_true")
    parser.add_argument("--agent-debug", action="store_true")
    parser.add_argument("--target", choices=["aarch64-apple-darwin", "x86_64-apple-darwin"])
    args = parser.parse_args()
    target = args.target or os.environ.get("CARGO_BUILD_TARGET")
    if target and target not in {"aarch64-apple-darwin", "x86_64-apple-darwin"}:
        parser.error("macOS packaging requires an Apple Darwin target")
    subprocess.run(["bun", "install", "--frozen-lockfile", "--ignore-scripts"], cwd=ROOT / "web", check=True)
    subprocess.run(["bun", "run", "build"], cwd=ROOT / "web", check=True)
    command = ["cargo", "build", "--locked", "-p", "reforge-bench-desktop"]
    if target:
        command.extend(["--target", target])
    if not args.debug:
        command.append("--release")
    if args.agent_debug:
        command.extend(["--features", "agent-debug"])
    build_env=os.environ.copy()
    if not args.debug:
        encoded=build_env.get("CARGO_ENCODED_RUSTFLAGS")
        flags=encoded.split("\x1f") if encoded else shlex.split(build_env.get("RUSTFLAGS",""))
        flags.extend([f"--remap-path-prefix={Path.home()}=/user",f"--remap-path-prefix={ROOT}=/reforge"])
        build_env["CARGO_ENCODED_RUSTFLAGS"]="\x1f".join(flags)
    subprocess.run(command, cwd=ROOT, env=build_env, check=True)
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"], cwd=ROOT))
    version = next(package["version"] for package in metadata["packages"] if package["name"] == "reforge-bench-desktop")
    receipt = {"version":version, "agent_debug":args.agent_debug, "debug":args.debug,
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
    binary = cache / ("debug" if args.debug else "release") / "reforge-bench-desktop"
    output = ROOT / "dist" / "Reforge Bench.app"
    output.parent.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(dir=output.parent) as temporary:
        bundle = Path(temporary) / output.name
        contents = bundle / "Contents"
        resources = contents / "Resources"
        (contents / "MacOS").mkdir(parents=True)
        resources.mkdir()
        shutil.copy2(binary, contents / "MacOS" / "reforge-bench-desktop")
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
        iconset = Path(temporary) / "AppIcon.iconset"
        iconset.mkdir()
        for size in [16, 32, 128, 256, 512]:
            for scale in [1, 2]:
                name = f"icon_{size}x{size}{'@2x' if scale == 2 else ''}.png"
                subprocess.run(["sips", "-z", str(size * scale), str(size * scale), str(ROOT / "desktop/assets/app-icon.png"), "--out", str(iconset / name)], check=True, stdout=subprocess.DEVNULL)
        subprocess.run(["/usr/bin/iconutil", "-c", "icns", str(iconset), "-o", str(resources / "AppIcon.icns")], check=True)
        info = {"CFBundleName": "Reforge Bench", "CFBundleDisplayName": "Reforge Bench", "CFBundleIdentifier": "dev.reforge.bench", "CFBundleExecutable": "reforge-bench-desktop", "CFBundlePackageType": "APPL", "CFBundleIconFile": "AppIcon", "CFBundleShortVersionString": version.split("-")[0], "CFBundleVersion": "1", "NSHighResolutionCapable": True, "LSMinimumSystemVersion": "13.0"}
        (contents / "Info.plist").write_bytes(plistlib.dumps(info))
        (resources / "build.json").write_text(json.dumps(receipt) + "\n")
        shutil.copy2(ROOT / "LICENSE", resources / "LICENSE")
        shutil.copytree(ROOT / "experiments/rune/licenses", resources / "licenses")
        subprocess.run(["codesign", "--force", "--deep", "--sign", "-", str(bundle)], check=True)
        if output.exists():
            shutil.rmtree(output)
        shutil.move(bundle, output)
    print(output)


if __name__ == "__main__":
    main()
