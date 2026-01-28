# Omics Upload Endpoint

Rust-based microservice that receives omics files via POST and stores them.

## Endpoints
- `GET /omics/health`
- `POST /omics/upload`
  - Body: raw bytes (any file)
  - Optional header: `X-Filename`

## Running locally
```bash
cargo run
