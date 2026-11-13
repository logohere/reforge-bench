# Host UI components

The locally built `dist/ui-components.js` includes selected modules from IBM's
`@carbon/web-components` 2.67.0, with Carbon icons/helpers and Lit runtime code.
Exact dependency versions and integrity values are retained in `web/bun.lock`.
Copyright notices and license terms are retained in this directory and the
generated bundle's legal notices.

- Carbon Web Components, icons and icon helpers: IBM Corp.; Apache-2.0.
- Lit, LitElement, lit-html and ReactiveElement: Google LLC; BSD-3-Clause.

Source: https://github.com/carbon-design-system/carbon/tree/main/packages/web-components
and https://github.com/lit/lit.

Imported entry modules: button, text-input, select, select-item, ui-shell/header,
ui-shell/header-menu-button, ui-shell/side-nav-items and ui-shell/side-nav-link. This does not claim
certification or endorsement by IBM. The separately retained local icon set has
its own source receipt in `ui/icons/`.

The optional electrical-simulation worker includes `eecircuit-engine` 1.9.0
(MIT, EElab.dev). Its notice is retained in `EEcircuit-engine-LICENSE`.
Project source: https://github.com/eelab-dev/EEcircuit-engine.

The hardware-scene bundle includes uPlot 1.6.32 (MIT, Leon Sorokin) for the
pulse-loop trace renderer. Its notice is retained in `uPlot-LICENSE`.
Project source: https://github.com/leeoniya/uPlot.

The source-excerpt bundle includes `@lezer/cpp` 1.1.6 and the existing Lezer
highlight/parser runtime (MIT, Marijn Haverbeke and contributors). Notices are
retained in `Lezer-cpp-LICENSE`, `Lezer-highlight-LICENSE`, `Lezer-common-LICENSE`
and `Lezer-lr-LICENSE`. Syntax coloring is not compilation or source validation.
Project sources: https://code.haverbeke.berlin/lezer/cpp and
https://code.haverbeke.berlin/lezer/highlight.

The app includes `@noble/hashes` 2.4.0 (MIT, Paul Miller) as a SHA-256 fallback
when Web Crypto is unavailable on a temporary HTTP LAN preview. Native Web
Crypto remains preferred; exact firmware, job and CAD identity checks remain
required. Its notice is retained in `Noble-hashes-LICENSE`.
Project source: https://github.com/paulmillr/noble-hashes.
