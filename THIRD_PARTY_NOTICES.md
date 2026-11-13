# Third-party notices

The project source and original example content use the root MIT license.
Dependencies keep their own licenses; project licensing does not relicense them.
No OEM firmware, vendor screenshots or vendored analysis engine are included.

## Desktop and hardware assets

- Rune and Rune Kit: MIT; pinned Git revision in `Cargo.lock`.
- Carbon icons and components: Apache-2.0; notices in
  `experiments/rune/licenses` and `web/ui/licenses`.
- Three.js, uPlot, Noble hashes, Lezer and the circuit engine retain their
  upstream notices in `web/THREE-LICENSE.txt` and `web/ui/licenses`.
- Pico 2 base-board CAD: MIT, © 2025 Raspberry Pi Ltd. Source receipt and full
  license accompany `web/assets/hardware/pico-2/model.glb`.
- Digilent Cora Rev B pin reference: MIT, © 2021 Digilent, Inc.; full license
  retained in the XML header under `hardware/cora/boards`.

Other device geometry is simplified bench artwork. External manufacturer and
schematic links are references, not bundled firmware or redistributed documents.

## Locked dependencies

| Component | Version | Declared license |
| --- | --- | --- |
| serde / serde_core / serde_derive | 1.0.229 | MIT OR Apache-2.0 |
| serde_json | 1.0.154 | MIT OR Apache-2.0 |
| itoa | 1.0.19 | MIT OR Apache-2.0 |
| memchr | 2.8.3 | Unlicense OR MIT |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 |
| quote | 1.0.47 | MIT OR Apache-2.0 |
| syn | 3.0.10 | MIT OR Apache-2.0 |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| zmij | 1.0.23 | MIT |
| Dotdog, development tooling | 1.0.0 | MIT |

`Cargo.lock` and `package-lock.json` identify the registry versions. Dependencies
are obtained from their registries rather than copied into this source archive.
Their installed packages contain upstream license/copyright notices. Preserve
the required notices in distributions.

Before distributing any binary or vendored dependency, generate and review its
complete license bundle. This source-only inventory is not that binary bundle.
