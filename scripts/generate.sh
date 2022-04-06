#!/bin/bash
# requires Lato font

img=$1
caption="$(echo "$2" | fold -sw 34)"
author="$(echo "$3" | sed 's/.*/\U&/')"
track_name="$(echo "$4" | sed 's/.*/\U&/')"
info="$author \\\"$track_name\\\""

out="${5:-out.png}"

if ! [[ -f "$1" ]] || [ $# -lt 4 ] || [ $# -gt 5 ]; then
	echo "Usage $0 cover.png caption author track_name [out.png]"
	exit 1
fi

cwd="$(pwd)"
dir=$(mktemp -d)
cp scripts/quote.png $dir
cp "$img" $dir
cd $dir

# scale the image and remove any metadata
if [ $(identify -format '%w' "$img") -gt $(identify -format '%h' "$img") ]; then
	convert $img +profile "*" -resize x600 resized.png
else
	convert $img +profile "*" -resize 900x resized.png
fi

# crop the image to be 900x600
convert resized.png -gravity center -extent 900x600 cropped.png

# darken the image
convert cropped.png -brightness-contrast -20x-25 darkened.png

# add author
convert darkened.png -gravity SouthWest \
	-pointsize 32 -fill white -font Lato-Bold \
	-draw "text 90,52 \"$info\"" captioned.png

i=$(( $(echo "$caption" | wc -l)-1 ))

# add quote symbol to first line
composite -geometry 42x42+25+$((405-$i*80)) quote.png captioned.png captioned.png

echo "$caption" | while read bar ; do
	# create text
	convert -background white -bordercolor white \
		-pointsize 50 -fill white -font Lato-Bold \
		-gravity South label:"$bar" -extent x68 -border 5x0 \
		-fill black -annotate +0+5 "$bar" \
		caption.png

	# add bar
	convert captioned.png caption.png -geometry +90+$((405-$i*80)) -composite captioned.png
	i=$(($i-1))
done 


# compress
# convert captioned.png -strip -interlace Plane -gaussian-blur 0.02 -quality 75%  "$out"
cp captioned.png "$out"

cp "$out" "$cwd"
cd "$cwd"
rm -r $dir
