import aiosqlite
from contextlib import asynccontextmanager
from app.config import settings


async def init_db():
    async with aiosqlite.connect(settings.db_path) as db:
        await db.executescript("""
            CREATE TABLE IF NOT EXISTS images (
                id INTEGER PRIMARY KEY,
                source_path TEXT NOT NULL UNIQUE,
                filename TEXT NOT NULL,
                scan_id TEXT,
                file_size INTEGER,
                width INTEGER,
                height INTEGER,
                year INTEGER,
                month INTEGER,
                title TEXT,
                status TEXT NOT NULL DEFAULT 'source',
                organized_path TEXT,
                enhanced_path TEXT,
                thumbnail_path TEXT,
                created_at TEXT DEFAULT (datetime('now')),
                updated_at TEXT DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY,
                status TEXT NOT NULL DEFAULT 'pending',
                steps TEXT NOT NULL,
                total_images INTEGER DEFAULT 0,
                completed_images INTEGER DEFAULT 0,
                failed_images INTEGER DEFAULT 0,
                error_message TEXT,
                created_at TEXT DEFAULT (datetime('now')),
                started_at TEXT,
                completed_at TEXT
            );

            CREATE TABLE IF NOT EXISTS task_items (
                id INTEGER PRIMARY KEY,
                task_id INTEGER NOT NULL REFERENCES tasks(id),
                image_id INTEGER NOT NULL REFERENCES images(id),
                status TEXT NOT NULL DEFAULT 'pending',
                current_step TEXT,
                error_message TEXT,
                started_at TEXT,
                completed_at TEXT
            );

            CREATE TABLE IF NOT EXISTS image_history (
                id INTEGER PRIMARY KEY,
                image_id INTEGER NOT NULL REFERENCES images(id) ON DELETE CASCADE,
                step TEXT NOT NULL,
                created_at TEXT DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
        """)
        await db.commit()
        try:
            await db.execute("ALTER TABLE images ADD COLUMN phash TEXT")
            await db.commit()
        except Exception:
            pass


@asynccontextmanager
async def get_db():
    async with aiosqlite.connect(settings.db_path) as db:
        db.row_factory = aiosqlite.Row
        yield db
