from contextlib import asynccontextmanager
from fastapi import FastAPI
from app.db import init_db
from app.config import settings
from app.scanner import scan_source
from app.routers import images
from app.routers import processing
from app.routers import ws
import os


@asynccontextmanager
async def lifespan(app: FastAPI):
    os.makedirs(settings.output_dir, exist_ok=True)
    await init_db()
    await scan_source()
    yield


app = FastAPI(lifespan=lifespan)

app.include_router(images.router)
app.include_router(processing.router)
app.include_router(ws.router)


@app.get("/api/health")
async def health():
    return {"status": "ok"}


@app.post("/api/scan")
async def rescan():
    count = await scan_source()
    return {"scanned": count}
