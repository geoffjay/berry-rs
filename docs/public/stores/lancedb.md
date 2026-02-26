# LanceDB

LanceDB is an embedded vector database — no server is required.

## Viewing Data

To view the contents of your LanceDB database, use the following Docker image:

```bash
docker pull ghcr.io/gordonmurray/lance-data-viewer:lancedb-0.24.3
chmod -R o+rx ~/Library/Application\ Support/berry/lancedb
docker run --rm -p 8080:8080 \
  -v ~/Library/Application\ Support/berry/lancedb:/data:ro \
  ghcr.io/gordonmurray/lance-data-viewer:lancedb-0.24.3
```

Access the viewer at http://localhost:8080.
