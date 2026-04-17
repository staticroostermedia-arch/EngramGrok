FROM python:3.11-slim

WORKDIR /app

# Install dependencies before copying source code to cache the heavy torch install
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Download the model weights during the Docker build process so the container 
# boots instantly offline without hanging to pull from huggingface hub.
RUN python -c "from sentence_transformers import SentenceTransformer; SentenceTransformer('all-MiniLM-L6-v2')"

# Copy the actual API source code
COPY server.py .

EXPOSE 8000

# Run the FastAPI server via uvicorn
CMD ["python", "server.py"]
