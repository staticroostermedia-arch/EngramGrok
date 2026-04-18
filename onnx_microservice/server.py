from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from sentence_transformers import SentenceTransformer
import os
import uvicorn

app = FastAPI(title="Engram ONNX Semantic Broker")

MODEL_NAME = os.getenv("MODEL_NAME", "all-MiniLM-L6-v2")
print(f"[ONNX] Loading SentenceTransformer: {MODEL_NAME}...")
model = SentenceTransformer(MODEL_NAME)
print("[ONNX] Model ready.")

# ── Legacy endpoint (internal use) ──────────────────────────────────────────
class EmbedRequest(BaseModel):
    text: str

class EmbedResponse(BaseModel):
    vector: list[float]

@app.post("/embed", response_model=EmbedResponse)
def embed_text(payload: EmbedRequest):
    if not payload.text or not payload.text.strip():
        raise HTTPException(status_code=400, detail="Empty text provided")
    embedding = model.encode(payload.text)
    return EmbedResponse(vector=embedding.tolist())

# ── OpenAI-compatible endpoint (used by ENGRAM_EMBED_URL) ───────────────────
class OAIEmbedInput(BaseModel):
    input: str | list[str]
    model: str = MODEL_NAME

class OAIEmbedData(BaseModel):
    object: str = "embedding"
    index: int
    embedding: list[float]

class OAIEmbedResponse(BaseModel):
    object: str = "list"
    model: str
    data: list[OAIEmbedData]

@app.post("/v1/embeddings", response_model=OAIEmbedResponse)
def openai_embed(payload: OAIEmbedInput):
    texts = payload.input if isinstance(payload.input, list) else [payload.input]
    if not texts or not any(t.strip() for t in texts):
        raise HTTPException(status_code=400, detail="Empty input")
    embeddings = model.encode(texts)
    data = [OAIEmbedData(index=i, embedding=e.tolist()) for i, e in enumerate(embeddings)]
    return OAIEmbedResponse(model=MODEL_NAME, data=data)

@app.get("/health")
def health():
    return {"status": "ready", "model": MODEL_NAME}

if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8089, log_level="warning")
