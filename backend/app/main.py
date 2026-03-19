from contextlib import asynccontextmanager
from fastapi import FastAPI
from app.db import init_db
from app.config import settings
import os


@asynccontextmanager
async def lifespan(app: FastAPI):
    os.makedirs(settings.output_dir, exist_ok=True)
    await init_db()
    yield


app = FastAPI(lifespan=lifespan)


@app.get("/api/health")
async def health():
    return {"status": "ok"}
