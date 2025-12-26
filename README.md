# Perp Screener

## Setup

```bash
conda create -n perpscreener python=3.11
conda activate perpscreener
pip install -r requirements.txt
cp .env.sample .env
```

## Run

```bash
uvicorn app.main:app --reload
```

## Endpoints

- `GET /health` - Health check
- `GET /greet/{name}` - Example greeting endpoint
