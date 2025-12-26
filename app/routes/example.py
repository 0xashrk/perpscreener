from fastapi import APIRouter

from app.services.example_service import get_greeting

router = APIRouter()


@router.get("/greet/{name}")
def greet(name: str):
    return get_greeting(name)
