# Use Reforge for your project

The included scope summary and pulse fixture are examples, not required hardware
or a connected lab. The native shell, artifact library, evidence graph, waveform
inspection and diagnostics can be used with your own inputs.

## Artifacts and evidence

Use **File → Import artifact** for your files. Imports retain file identity and
stay local; they do not execute firmware or invoke an analysis engine.
Import a [Dotdog graph export](rea-maps.md) for your components, pins, functions
and evidence relationships. Its producer identities and declared limits remain
attached. Reforge does not infer that an imported graph belongs to another file.

## Equipment

In **Hardware → Equipment**, edit the equipment name, role and interface; add or
remove devices. **Import list** and **Export list** use the
[equipment schema](../web/hardware/equipment.schema.json). Lists have at most
64 devices and 64 KiB. Unknown imported fields are discarded; exports contain
only the documented equipment fields, not transport credentials or application
settings. Do not put credentials in free-text fields.

Equipment records are interchangeable inventory data. They do not relabel a
different board as electrically compatible. The detailed 2D/3D geometry, pin
inspection, wiring guide and RTL replay remain the labeled example preset.
Different physical boards require their own models and validated pin mapping;
changing an equipment record does not change those models or program a device.

## Captures and adapters

**Hardware → Captures** reads supported waveform files without a live adapter.
Inspect the waveform or export its identity receipt. Comparing against the
example's 1 kHz pulse is optional; other projects are not judged against it.
Retain the original capture and its acquisition settings.

Live acquisition and analysis integrations are user-supplied. No particular
computer, remote account, SSH setup, firmware library or analysis engine is
required or provisioned. Capture controls remain unavailable until an adapter
supplies a real connection and valid frames.

## Extend the reference implementation

- Equipment interchange: `web/hardware/equipment-config.js` and its schema.
- Geometry and wiring preset: `web/hardware/layout.js`, `plan.js`, `tabletop.js`
  and the device model modules. Preserve physical units and source attribution.
- Waveform parsing: `web/hardware/scope-capture.js`.
- Evidence mapping: `dotdog.rs`, `graph_model.rs`, `graph.rs` in the native UI crate.
- Native data/adapter boundary: `provider.rs` and `hardware_host.rs`.

Keep project data separate from reference assets and executable adapters. Verify
the affected import, failure states and rendered views before claiming support
for a new device. The supplied models are not a universal hardware plugin API.
