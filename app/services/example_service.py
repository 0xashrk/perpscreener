def get_greeting(name: str) -> dict:
    """Business logic for generating a greeting."""
    return {
        "message": f"Hello, {name}!",
        "name": name
    }
