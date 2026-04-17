from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from sentence_transformers import SentenceTransformer
import os
import uvicorn

app = FastAPI(title="Engram ONNX Semantic Broker")

# Load the model once on startup and pin it to CPU/RAM.
# all-MiniLM-L6-v2 outputs a 384-dimensional normalized vector.
MODEL_NAME = os.getenv("MODEL_NAME", "all-MiniLM-L6-v2")
print(f"[ONNX] Loading Sentencetransformer: {MODEL_NAME}...")
model = SentenceTransformer(MODEL_NAME)
print("[ONNX] Model resident in memory. Ready for semantic transduction.")

class EmbedRequest(BaseModel):
    text: str

class EmbedResponse(BaseModel):
    vector: list[float]

@app.post("/embed", response_model=EmbedResponse)
def embed_text(payload: EmbedRequest):
    if not payload.text or not payload.text.strip():
        raise HTTPException(status_code=400, detail="Empty text provided")
    
    # Fast ONNX inference bypasses GPU context switching
    embedding = model.encode(payload.text)
    
    # Return as standard JSON f32 array 
    # (The Rust host handles the f32 -> INT8 quantization before passing to VRAM)
    return EmbedResponse(vector=embedding.tolist())

if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8000, log_level="warning")
