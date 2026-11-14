"""Build the native application with its own resources and macOS identity."""

from pathlib import Path
import argparse
import json
import os
import plistlib
import shutil
import subprocess
import tempfile

from package_common import ROOT, build_desktop, copy_resources


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--debug", action="store_true")
    parser.add_argument("--agent-debug", action="store_true")
    parser.add_argument("--target", choices=["aarch64-apple-darwin", "x86_64-apple-darwin"])
    args = parser.parse_args()
    target = args.target or os.environ.get("CARGO_BUILD_TARGET")
    if target and target not in {"aarch64-apple-darwin", "x86_64-apple-darwin"}:
        parser.error("macOS packaging requires an Apple Darwin target")
    binary, receipt = build_desktop(args.debug, args.agent_debug, target)
    version = receipt["version"]
    output = ROOT / "dist" / "Reforge Bench.app"
    output.parent.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(dir=output.parent) as temporary:
        bundle = Path(temporary) / output.name
        contents = bundle / "Contents"
        resources = contents / "Resources"
        (contents / "MacOS").mkdir(parents=True)
        resources.mkdir()
        shutil.copy2(binary, contents / "MacOS" / "reforge-bench-desktop")
        copy_resources(resources)
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
        subprocess.run(["codesign", "--force", "--deep", "--sign", "-", str(bundle)], check=True)
        if output.exists():
            shutil.rmtree(output)
        shutil.move(bundle, output)
    print(output)


if __name__ == "__main__":
    main()
