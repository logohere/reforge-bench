# REA evidence maps with Dotdog

Dotdog supplies graph preparation and queries for firmware, hardware and evidence
relationships. Reforge imports saved Dotdog JSON into its native graph inspector.
It does not bundle an analysis engine or launch an external provider.

## Open a map

Export an existing Dotdog graph:

```sh
dotdog graph export --dag .doghouse/imported/investigation.dag --format json > investigation.json
```

Open the export through Library → Open firmware, complete the import, then choose
REA Tool → Evidence map. It is recognized as a Dotdog graph, not executable
firmware. The default case's Evidence map contains the hardware family reference.
Search nodes, select relationships and inspect incoming/outgoing links and
provenance. Hardware retains the corresponding components and pins.

## Identities and limits

The importer accepts Dotdog 1.0 JSON with stable node IDs and relationships using
`sourceId`, `targetId` and `verb`. Limits: 16 MiB, 16,384 nodes and 32,768 edges.
Duplicate identities and unresolved links are rejected. Original node properties,
confidence, origin and declared limitations remain alongside the export SHA-256.

The library identity is the export hash, not an inferred binary identity.
Addresses and binary hashes remain producer-supplied evidence. There is no
automatic source-to-binary reconciliation. An absent graph path does not prove
independence; a logical pin relationship is not measured voltage or timing.

Bring your own compatible analysis/export workflow. No raw proprietary firmware,
private sessions or access configuration are supplied.

The separate `npm run model:*` commands map this repository for contributors;
they are not the REA investigation and do not prove runtime coverage.
