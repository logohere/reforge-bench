# Preparing a public source release

Publication requires the owner's explicit approval. The MIT-licensed source is
an alpha preview for local macOS and Windows x64 builds, not a supported stable desktop release.

Source builds do not require an Apple developer account, Developer ID signing
or notarization. The source preview and a public macOS binary have separate gates.
Local ad-hoc signing is not Developer ID signing or notarization. Before a public
binary release, verify the packaged app, its complete third-party license bundle,
Developer ID signature and notarization on a clean supported machine. Confirm
that ordinary builds leave the developer inspector disabled.

Windows source builds require no signing account. Windows packages are unsigned
folders/ZIPs, not installers. Before a public Windows binary release, verify
WebView2 availability, runtime dependencies, Authenticode signing and packaged
resources on a clean supported machine. CI smoke screenshots are not a complete
clean-machine or hardware qualification.

## Review gates

1. Review every tracked file and all Git history, including branches and tags.
   Inspect issue/PR text, screenshots, logs, build outputs and package contents.
   Keep private review material outside the repository.
2. Run the distribution check and the configured Gitleaks rules, which extend
   the defaults with home-path and private-network checks. Review findings;
   do not silence a failing check with a blanket allowlist.
3. Verify licenses and attribution for code, data, images, fonts and models.
   Owning a device or downloading firmware does not grant redistribution rights.
   Review the project name for conflicts before launch; the working name is not
   a trademark clearance.
4. Keep hardware/analysis configuration user-supplied. Qualify debug access and
   diagnostic redaction before including those systems in a release.
5. Run the documented checks and relevant rendered/hardware acceptance. Record
   limitations and use semantic versions; prerelease work is not a stable release.
   Distinguish local checks from hosted CI and clean-machine execution. Document
   unexecuted checks for a source preview; require successful architecture-specific
   hosted checks before claiming verified desktop release support.

## Source packaging

From a clean, reviewed commit:

```sh
python3 scripts/check-distribution.py
gitleaks git --redact --log-opts='--all --full-history' .
python3 scripts/package-source.py
```

The package script uses `git archive HEAD`, includes a commit identity in the
filename, and does not copy the working directory, Git database or ignored files.
Do not publish a ZIP of a development folder. Review the resulting archive before
uploading; this command does not publish anything.
