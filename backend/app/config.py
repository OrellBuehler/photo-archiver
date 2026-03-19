from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    source_dir: str = "/data/source"
    output_dir: str = "/data/output"
    db_path: str = ""
    thumbnail_size: int = 400
    device: str = "cpu"

    def model_post_init(self, __context):
        if not self.db_path:
            self.db_path = f"{self.output_dir}/photo-archiver.db"


settings = Settings()
