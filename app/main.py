from fastapi import FastAPI

from app.core.config import settings
from app.routes import health, example

app = FastAPI(title=settings.app_name, debug=settings.debug)

app.include_router(health.router)
app.include_router(example.router)
