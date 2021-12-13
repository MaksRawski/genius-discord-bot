#!/bin/bash

# Requirements:
# - curl
# - jq
# - fzf
# - GENIUS_TOKEN (CLIENT ACCESS TOKEN from https://genius.com/api-clients)

echo -n "Type song, album or artist name: "
read query

query_json=$(curl -s -G \
	-H "Authorization: Bearer $GENIUS_TOKEN" \
	--data-urlencode "q=$query" \
	https://api.genius.com/search
)

result=$(echo "$query_json" | jq '.response.hits[] | "\(.result.id): \(.result.artist)-\(.result.title)"' -r | fzf --layout=reverse)
song_id=$(echo $result | cut -d : -f 1)
song_name=$(echo $result | cut -d : -f 2 | xargs)

img=$(curl -s -H "Authorization: Bearer $GENIUS_TOKEN" \
	https://api.genius.com/songs/$song_id | jq '.response.song.header_image_url' -r
)

curl -s "$img" -o "$song_name.jpg"
