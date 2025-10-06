# Discord Genius Bot
Discord bot to create genius-like cards + some other genius related utilities.

# Usage

## Discord bot
The entire project is written in Rust, so can be simply compiled with

```
cargo build --release
```

To run it you will need to set `GENIUS_TOKEN` and a `DISCORD_TOKEN` environmental variables.
For `GENIUS_TOKEN` you will first need to get a `client access token`, which you can get from here: https://genius.com/api-clients.
For `DISCORD_TOKEN` you will have to [create a Discord application](https://discord.com/developers/applications/) and get the TOKEN from the Bot tab.

Then run with:

```
export GENIUS_TOKEN=abcef
export DISCORD_TOKEN=ghijkl
./target/release/genius # or move it wherever you want since it's statically linked
```

### Docker
Additionally I provide a `Dockerfile` in case you want to run it e.g. on your desktop in a background.
In that case do the following:

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

Logs can be previewed with:
```
docker logs genius
```

## Scripts
Initially I just wanted to have a way of creating such lyric cards so I wrote bash scripts, 
which are still available in `scripts/` folder.

### Donwloading song art images
Since we're trying to mimick Genius cards why not get images from there?
To donwload an image from Genius you will first need to create a Genius account and obtain a 
`client access token`, which you can get from here: https://genius.com/api-clients.
Save it in an environmental variable `GENIUS_TOKEN` and then run `genius-dl.sh` script,
which will allow you to search for a song and later with `fzf` chose a result.

```
export GENIUS_TOKEN=abcdef
./scripts/genius-dl.sh
```

`genius-dl.sh` has the following requirements:
- `curl`
- `jq`
- `fzf`
- `GENIUS_TOKEN`

### Generating cards
```
Usage ./scripts/generate.sh cover.png caption author track_name [out.png]
```

`generate.sh` has the following requirements:
- `ImageMagick`
- Lato font
