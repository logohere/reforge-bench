"""Build a Windows x64 desktop folder and ZIP, without signing or publishing."""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

from package_common import ROOT, build_desktop, copy_resources


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--debug", action="store_true")
    parser.add_argument("--agent-debug", action="store_true", help="Enable WebView2 developer tools; no socket inspector")
    args = parser.parse_args()
    if os.name != "nt":
        parser.error("Build the Windows application on Windows with the MSVC toolchain")
    target = os.environ.get("CARGO_BUILD_TARGET") or subprocess.check_output(
        ["rustc", "--print", "host-tuple"], cwd=ROOT
    ).decode().strip()
    if target != "x86_64-pc-windows-msvc":
        parser.error("Windows packaging currently supports x86_64-pc-windows-msvc only")
    binary, receipt = build_desktop(args.debug, args.agent_debug, target)
    directory = ROOT / "dist"
    output = directory / "Reforge Bench"
    if directory.is_symlink() or output.is_symlink():
        raise SystemExit("Distribution directories must not be symlinks.")
    directory.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(dir=directory) as temporary:
        bundle = Path(temporary) / output.name
        resources = bundle / "resources"
        resources.mkdir(parents=True)
        shutil.copy2(binary, bundle / "reforge-bench-desktop.exe")
        copy_resources(resources)
        (resources / "build.json").write_text(json.dumps(receipt) + "\n", encoding="utf-8")
        archive = shutil.make_archive(
            str(Path(temporary) / "package"), "zip", root_dir=temporary, base_dir=output.name
        )
        if output.exists():
            shutil.rmtree(output)
        shutil.move(bundle, output)
        archive_name = f"reforge-bench-{receipt['version']}-windows-x64.zip"
        os.replace(archive, directory / archive_name)
    print(output)
    print(directory / archive_name)


if __name__ == "__main__":
    main()
