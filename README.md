# Omics Upload Endpoint

Rust-based microservice that receives omics files via POST and stores them.

## Endpoints
- `GET /omics/health`
- `POST /omics/upload`
  - Body: raw bytes (any file)
  - Optional header: `X-Filename`

# Local development
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
docker compose -f dev/sender-compose.yaml build
docker compose -f dev/sender-compose.yaml up
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

## Build and run the data lake (receiver)
```bash
docker compose -f dev/receiver-compose.yaml build
docker compose -f dev/receiver-compose.yaml up
```
- [development setup](dev)
- [development omics ingest](dev/sender-compose.yaml)
- [development data lake](dev/reciever-compose.yml)

Notes:
- Docker builds must use the workspace root as the build context
- Dev Dockerfiles rely on the Cargo workspace layout
- Environment variables are defined in the compose files

Stop services:
```bash
docker compose -f dev/sender-compose.yaml down
docker compose -f dev/receiver-compose.yaml down
```
