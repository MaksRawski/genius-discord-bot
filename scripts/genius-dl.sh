#!/bin/bash

# Requirements:
# - curl
# - jq
# - fzf
# - GENIUS_TOKEN (CLIENT ACCESS TOKEN from https://genius.com/api-clients)

set -e
trap 'echo Error on line $LINENO ' ERR

if [ ! -v GENIUS_TOKEN ]; then
	echo "GENIUS_TOKEN not set, exiting."
	exit 1;
fi

echo -n "Type song, album or artist name: "
read query

query_json=$(curl -s -G \
	-H "Authorization: Bearer $GENIUS_TOKEN" \
	--data-urlencode "q=$query" \
	https://api.genius.com/search
)

echo "$query_json" > a.json
result=$(echo "$query_json" | jq '.response.hits[] | "\(.result.id): \(.result.artist_names) - \(.result.title)"' -r | fzf)
song_id=$(echo "$result" | cut -d : -f 1)

song_json=$(curl -s -H "Authorization: Bearer $GENIUS_TOKEN" https://api.genius.com/songs/$song_id)
echo "$song_json" > asdf.json
img_url=$(echo "$song_json" | jq '.response.song.song_art_image_url' -r)

ext="${img_url##*.}"
file_name="$(echo $song_json | jq '.response.song.artist_names + " - " + .response.song.title + ".'"$ext"'"' -r)"
curl "$img_url" -o "$file_name"
echo Succesfully saved as "$file_name"
