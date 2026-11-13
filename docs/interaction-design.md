# Workbench interaction conventions

Preserve the native workspace shell and shared tokens. Keep the current artifact,
its evidence and the hardware reference visible without adding another dashboard.

- Put one useful next action beside the selected item. Keep provenance and
  limitations available in disclosures; never turn unknown into passed.
- Separate artifact import, equipment records, reference models, replay and live
  acquisition. Equipment names do not configure drivers or electrical mappings.
- Show only working actions. An unavailable adapter needs a clear setup path,
  not a control that implies a connection or operation succeeded.
- Keep physical geometry proportional. On narrow windows, stack supporting
  controls and use one document scroll owner where practical.
- Preserve edits and view state during navigation. Errors must leave retained
  evidence available and offer a relevant recovery action.
- Use native menus and keyboard access; developer diagnostics stay secondary to
  bench work. Never upload reports automatically.

## Workflow references

These are interaction references, not dependencies or copied implementations.

[WaveForms](https://files.digilent.com/manuals/WaveForms/3.24.3/main.html)
separates workspace and instrument state, retains instrument settings, and
makes unsupported instruments unavailable. Reforge applies that distinction to
reference views and adapter-dependent acquisition.

[ImHex Data Inspector](https://docs.werwolv.net/imhex/views/data-inspector)
keeps byte interpretation tied to the selected data and allows irrelevant rows
to be hidden. Its documentation distinguishes a possible decoding from proven
meaning. Reforge similarly keeps evidence detail secondary and unknowns explicit.

[OpenHTF](https://github.com/google/openhtf) separates test phases, measurements,
attachments and equipment plugs. For Reforge, keep evidence records distinct from
device-specific adapters; do not hardcode a contributor workstation into a bench.
