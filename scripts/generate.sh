#!/bin/bash
# requires Lato font

set -e
trap 'echo Error on line $LINENO ' ERR

img="$1"
caption="$(echo "$2" | fold -sw 34)"
author="$(echo "$3" | perl -pe 'use utf8; $_=uc')"
track_name="$(echo "$4" | perl -pe 'use utf8; $_=uc')"
info="$author \"$track_name\""
font="Lato-Bold"

out="${5:-out.png}"

if [ $# -lt 4 ] || [ $# -gt 5 ]; then
	echo "Usage $0 cover.png caption author track_name [out.png]"
	exit 1
fi

if [ ! -f "$1" ]; then
	echo "No such file: $1"
	exit 1
fi

output_dir="$(pwd)"
script_dir=$(dirname "$(readlink -f "$0")")
tmp_dir=$(mktemp -d)
if [ ! -f "$script_dir/quote.png" ]; then
	echo "quote.png must be in the same directory as generate.sh!"
	exit 1
fi
cp "$script_dir/quote.png" "$tmp_dir"
input_img="$tmp_dir/input.${img##*.}"
cp "$img" "$input_img"
cd "$tmp_dir"

MAGICK_CMD=magick
if ! command -v $MAGICK_CMD &> /dev/null; then
	MAGICK_CMD=convert
fi
if ! command -v $MAGICK_CMD &> /dev/null; then
	echo "ImageMagick not installed!"
	exit 1
fi

# scale the image and remove any metadata
if [ $(identify -format '%w' "$input_img") -gt $(identify -format '%h' "$input_img") ]; then
	$MAGICK_CMD "$input_img" +profile "*" -resize x600 resized.png
else
	$MAGICK_CMD "$input_img" +profile "*" -resize 900x resized.png
fi

# crop the image to be 900x600
$MAGICK_CMD resized.png -gravity center -extent 900x600 cropped.png

# darken the image
$MAGICK_CMD cropped.png -brightness-contrast -20x-25 darkened.png

# add author
$MAGICK_CMD darkened.png -gravity SouthWest \
	-pointsize 32 -fill white -font "$font" \
	-draw "text 90,52 '$info'" captioned.png

i=$(( $(echo "$caption" | wc -l)-1 ))

# add quote symbol to first line
composite -geometry 42x42+25+$((405-$i*80)) quote.png captioned.png captioned.png

echo "$caption" | while read bar ; do
	# create text
	$MAGICK_CMD -background white -bordercolor white \
		-pointsize 50 -fill white -font "$font" \
		-gravity South label:"$bar" -extent x68 -border 5x0 \
		-fill black -annotate +0+5 "$bar" \
		caption.png

	# add bar
	$MAGICK_CMD captioned.png caption.png -geometry +90+$((405-i*80)) -composite captioned.png
	i=$((i-1))
done 


# compress
# $MAGICK_CMD captioned.png -strip -interlace Plane -gaussian-blur 0.02 -quality 75% compressed.png
cp captioned.png "$output_dir/$out"
echo "Card generated succesfully! Saved as $out"

cd "$output_dir"
rm -r "$tmp_dir"
