# Discord Genius Bot
Discord bot to create genius-like cards + some other genius related utilities.

# Building

## Pre-builts
Pre-built binaries as well as docker images are available to download from the releases page.
To use them you will need to provide them your `.env` (with `DISCORD_TOKEN` and `GENIUS_TOKEN`).
To do that, with a container you will need to run:

```
docker cp .env genius:/genius/.env
```

In case of a binary make sure that you have: `genius` (binary), `.env` and `scripts/` all in the same folder.

## Locally to current arch
```
docker build -t genius-bot .
```

## To different one e.g. ARM (which is my main target)
I find buildx to work best, more in-depth build process can be found in `.gitlab-ci.yml`.


1. Create a builder for desired architecture:
```
docker buildx create --driver docker-container --platform linux/arm64 --use
```

2. Start the build:
```
docker buildx build --pull --platform linux/arm64 .
```
