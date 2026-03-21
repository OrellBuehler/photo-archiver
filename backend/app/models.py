from pydantic import BaseModel


class ImageOut(BaseModel):
    id: int
    source_path: str
    filename: str
    scan_id: str | None
    file_size: int | None
    width: int | None
    height: int | None
    year: int | None
    month: int | None
    title: str | None
    status: str
    organized_path: str | None
    enhanced_path: str | None
    thumbnail_path: str | None
    phash: str | None = None
    created_at: str | None
    updated_at: str | None


class ImageUpdate(BaseModel):
    year: int | None = None
    month: int | None = None
    title: str | None = None


class ImageListResponse(BaseModel):
    images: list[ImageOut]
    total: int
    page: int
    per_page: int


class ImageStats(BaseModel):
    year: int | None
    status: str
    count: int


class FilterCountItem(BaseModel):
    value: str | int | None
    count: int


class FilterCounts(BaseModel):
    years: list[FilterCountItem]
    months: list[FilterCountItem]
    statuses: list[FilterCountItem]
    steps: list[FilterCountItem]
    total: int


class TaskOut(BaseModel):
    id: int
    status: str
    steps: list[str]
    total_images: int
    completed_images: int
    failed_images: int
    error_message: str | None
    created_at: str | None
    started_at: str | None
    completed_at: str | None


class TaskCreate(BaseModel):
    image_ids: list[int] | str
    steps: list[str]


class TaskItemOut(BaseModel):
    id: int
    task_id: int
    image_id: int
    status: str
    current_step: str | None
    error_message: str | None
    started_at: str | None
    completed_at: str | None


class ImageHistoryOut(BaseModel):
    id: int
    image_id: int
    step: str
    created_at: str | None


class ProgressMessage(BaseModel):
    type: str
    task_id: int
    image_id: int | None = None
    step: str | None = None
    progress: float | None = None
    message: str | None = None


class BulkDeleteRequest(BaseModel):
    image_ids: list[int]


class BulkUpdateRequest(BaseModel):
    image_ids: list[int]
    year: int | None = None
    month: int | None = None
    title: str | None = None


class DuplicateGroup(BaseModel):
    images: list[ImageOut]
    distance: int


class AppSettings(BaseModel):
    source_dir: str
    output_dir: str
    thumbnail_size: int
    device: str


class AppSettingsUpdate(BaseModel):
    thumbnail_size: int | None = None
    device: str | None = None
