# Independent pulse fixture

Pico GP2 supplies a 1 kHz test input through TPH2 pin 1 to Cora JA1 / Y18.
Cora JA2 / Y19 supplies the divided response through TPH2 pin 2.
Use a common signal ground. Do not join regulator outputs or connect 5 V to Pmod
signals. The scope and analyzer observe the fixture independently.

The included Lab replay comes from RTL simulation, not a physical capture.
It covers nominal input, missing input, stuck-high input and reset. Source hashes
bind the replay to the included RTL and testbench; they do not qualify hardware.

Use the 2D/3D wiring guide to inspect connector numbering and signal routes.
Check your board revision, supplies, probe settings and toolchain before using
your own hardware. Configure your own acquisition adapter and authenticated
transport; no account, receiver path, SSH key or live connection is bundled.

Imported waveforms remain unverified file data until their provenance and scaling
are established. Scope and analyzer clocks are not synchronized by arrival time.
