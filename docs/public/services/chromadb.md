# ChromaDB

ChromaDB is a vector database for storing embeddings. It can be used by Berry to store memories.

## Install

```bash
docker run -d -p 8000:8000 chromadb/chroma
```

## Configure

Create a `.env` file in the root of your Berry project with the following contents:

```bash
CHROMA_URL=http://localhost:8000
CHROMA_API_KEY=your-api-key
```

## Pre-requisites

```bash
mkdir -p ~/.config/berry
cat > ~/.config/berry/config.jsonc << 'EOF'
{
  "chroma": {
    "url": "http://localhost:8000",
    "collection": "berry_memories"
  }
}
EOF
```
