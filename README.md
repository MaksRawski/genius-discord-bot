# Discord Genius Bot
Discord bot to create genius-like cards + some other genius related utilities.

Since version 0.2 this project is using ImageMagick 7.1 which isn't yet available on debian based systems.
Very hacky solution can be found in the `Dockerfile` and it is a recommended way to use this project right now.

# Usage
First build the project itself:
```
docker build -t genius-bot .
```

Then put your `.env` (containing `DISCORD_TOKEN` and `GENIUS_TOKEN`) into a new container:
```
docker create --name genius genius-bot
docker cp .env genius:/genius/.env
```

And finally run with:
```
docker start genius
```

## Logs
Logs can be previewed with:
```
docker logs genius
```

# TODO
- [ ] properly center text in bars:
`load_image("label:...")`

- [ ] use album/artist covers for the image background

