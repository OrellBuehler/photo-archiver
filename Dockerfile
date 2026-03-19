# Stage 1: Build frontend
FROM node:22-slim AS frontend-build
WORKDIR /app/frontend

RUN npm install -g bun

COPY frontend/package.json frontend/bun.lock* ./
RUN bun install --frozen-lockfile

COPY frontend/ ./
RUN bun run build

# Stage 2: Python runtime
FROM python:3.12-slim AS runtime

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    libimage-exiftool-perl \
    libgl1-mesa-glx \
    libglib2.0-0 \
    && rm -rf /var/lib/apt/lists/*

# Install uv
COPY --from=ghcr.io/astral-sh/uv:latest /uv /usr/local/bin/uv

WORKDIR /app

# Install Python dependencies
COPY backend/pyproject.toml backend/uv.lock ./backend/
RUN cd backend && uv sync --no-dev --frozen

# Copy backend code
COPY backend/ ./backend/

# Copy built frontend
COPY --from=frontend-build /app/frontend/build ./dist/

# Download model weights
RUN mkdir -p /data/output/.models && \
    cd backend && uv run python -c " \
from basicsr.utils.download_util import load_file_from_url; \
load_file_from_url('https://github.com/xinntao/Real-ESRGAN/releases/download/v0.1.0/RealESRGAN_x4plus.pth', model_dir='/data/output/.models'); \
load_file_from_url('https://github.com/TencentARC/GFPGAN/releases/download/v1.3.4/GFPGANv1.4.pth', model_dir='/data/output/.models'); \
" || echo "Model download skipped (will download on first use)"

EXPOSE 8000

CMD ["backend/.venv/bin/uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8000", "--app-dir", "backend"]
