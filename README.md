# Genomics Data Warehouse – Bridgehead Architecture

This repository documents and implements a secure, federated genomics data ingestion pipeline based on FHIR, pseudonymisation, and a central data warehouse.

The design follows a bridgehead pattern:
partners keep control over their data, while pseudonymised data is transferred securely into a central data warehouse for analytics and downstream services.

# High-Level Architecture

Flow (top → bottom):
1.	Partner pushes data into a local FHIR Blaze and the genetic data(MAF) endpoint
2.	Data is pseudonymised at the partner bridgehead
3.	Pseudonymised data is transferred via Samply BEAM
4.	Data lands in a central FHIR Blaze and genomic data in S3 store  
5.	Genomic data is transformed into Parquet/Iceberg for analytics
6.	Optional: exports for cBioPortal or other consumers

# Components

## Partner Bridgehead / Uploader / Ingest Service
![Ingest Service](/docu/Ingest-BridgeheadV2.png)

- FHIR Blaze (Partner)
- Receives FHIR resources pushed by the partner
- Stores identifiable data locally (never leaves the partner site)
- Pseudonymisation Service
- Read FHIR resources from Blaze
- Replaces patient/sample identifiers with deterministic pseudonyms
- No identifying data leaves the bridgehead
- Transfer Metadata

Samply BEAM
- Secure, auditable transport layer

Transfers:
- pseudonymised FHIR payloads (FHIR Bundles)
- pseudonymised MAF file
- metadata sidecars
- No interpretation of data, transport only

## Data Warehouse
![Data Warehouse](/docu/central-DWH-V2.png)

### Clinical Data
- FHIR Blaze (Central)
- Stores pseudonymised FHIR resources
- Acts as the canonical clinical/genomic API
### Genomic Data
- Raw Zone
- Immutable storage of received payloads
- Full auditability and replay capability
- Transformation of MAF → Parquet
Optional: Iceberg tables for:
- variants
- samples
- patients
- Optimised for analytics and large-scale queries
Exporter / APIs Generate:
- cBioPortal import packages (MAF + meta files)
- Analytics datasets

# Why This Architecture?

### Security & Governance
- Identifiable data never leaves the partner site
- Pseudonymisation happens before transfer
- Clear trust boundaries (Partner → BEAM → Central)

### Interoperability
- FHIR as the exchange and API format
- Works with clinical, genomic, and metadata resources
- Compatible with existing healthcare tooling

### Performance & Scalability
- Streaming ingestion
- Columnar storage (Parquet) for analytics
- Iceberg enables:
- snapshots
- reproducibility
- incremental updates

### Flexibility
- Partners can onboard independently
- Central services can evolve without impacting partners
- Supports multiple downstream consumers

⸻

### Data Formats

| Layer          | Format                 | Purpose                  |
|----------------|------------------------|--------------------------|
| Partner ingest | FHIR (JSON)            | Clinical exchange        |
| Partner ingest | MAF                    | genomic exchange         |
| Transfer Beam  | FHIR Bundle / MAF file | Secure transport         |
| Blaze FHIR     | Immutable files        | FHIR Data store          |
| Data warehouse | Parquet + Iceberg      | Analytics & querying     |
| Export         | MAF + meta files       | cBioPortal compatibility |

## Architects working with:
- FHIR
- GA4GH
- cBioPortal
- Samply BEAM

## Ingest Service / Data Warehouse
[Documentation Proposel](/docu/itcc.pdf)

### Endpoints
- `GET /omics/health`
- `POST /omics/upload`
  - Body: raw bytes (any file)
  - Optional header: `X-Filename`

# Development

## Local development
This repository is a Rust workspace with multiple services (itcc-omics-ingest and itcc-omics-data-lake).

### Requirements:
- Docker
- Docker Compose (v2)
## Beam
Samply.Beam is a distributed task broker designed for efficient communication across strict network environments. It provides most commonly used communication patterns across strict network boundaries, end-to-end encryption and signatures, as well as certificate management and validation on top of an easy to use REST API. In addition to task/response semantics, Samply.Beam supports high-performance applications with encrypted low-level direct socket connections.
- [Beam](https://github.com/samply/beam)
- [Beam file transfer](https://github.com/samply/beam-file)

## Getting started

Using Docker, you can run a small demo beam network by checking out the git repository (use `main` or `develop` branch) and running the following command:
```bash
./dev/beamdev --tag develop-sockets demo
```
This will launch your own beam demo network, which consists of one broker (listening on `localhost:8080`) and two connected proxies (listening on `localhost:8081` and `localhost:8082`).

The following paragraph show a MAF file transfer
using [beam-file](https://github.com/samply/beam-file) calls. Two parties (and their Samply.Proxies) are
connected via a central broker. Each party has one registered application.
In the next section we will simulate the communication between these applications over the beam network.

The used BeamIds are the following:

| System             | BeamID                       |
|--------------------|------------------------------|
| Broker             | broker                       |
| Proxy1             | proxy1.broker                |
| App behind Proxy 1 | app1.proxy1.broker           |
| Proxy2             | proxy2.broker                |
| App behind Proxy 2 | app2.proxy2.broker           |

To simplify this example, we use the same ApiKey `App1Secret` for both apps. Also, the Broker has a short name (`broker`) where in a real setup, it would be required to have a fully-qualified domain name as `broker1.samply.de` (see [System Architecture](#system-architecture)).

## Build and run the ingest service
```bash
docker compose -f dev/ingest-compose.yaml build
docker compose -f dev/ingest-compose.yaml up
```
The ingest service will be available at:
```bash
  http://localhost:6080
```
Endpoints
- `GET /omics/health`
- `POST /omics/upload`
  - Body: raw bytes (any file)
  - Optional header: `X-Filename`

## Build and run the data-warehouse (receiver)
```bash
docker compose -f dev/dwh-compose.yaml build
docker compose -f dev/dwh-compose.yaml up
```
- [development setup](dev)
- [development omics ingest](dev/ingest-compose.yaml)
- [development Data Warehouse](dev/dwh-compose.yaml)

Notes:
- Docker builds must use the workspace root as the build context
- Dev Dockerfiles rely on the Cargo workspace layout
- Environment variables are defined in the compose files

Stop services:
```bash
docker compose -f dev/ingest-compose.yaml down
docker compose -f dev/dwh-compose.yaml down
```
# Kubernetes
