#!/bin/bash
set -euo pipefail



# Step 1: convert geotiff to mbtiles

for tif in ./overlays/*_modified.tif; do
    # Strip extension and replace with .mbtiles
    mbtiles="${tif%_modified.tif}.mbtiles"

    echo "Processing $tif -> $mbtiles"
    rio mbtiles \
        --format WEBP \
        --zoom-levels 16..20 \
        --resampling lanczos \
        --rgba \
        --progress-bar \
        --exclude-empty-tiles \
        "./$tif" "./$mbtiles"
done

# Move to different directory
rm -f overlay_levels/*
mv overlays/*.mbtiles overlay_levels/

# Step 2: Merge per level
levels=$(ls ./overlay_levels/*.mbtiles | sed -E 's/.*_(.+)\.mbtiles/\1/' | sort -u)
echo "Merging mbtiles by levels: $levels"

for lvl in $levels; do
    echo "Processing level $lvl"
    input="$(ls ./overlay_levels/*_$lvl.mbtiles)"
    sources=$(echo "$input" | tr '\n' ',' | sed 's#./overlay_levels/##g' | sed 's#\.mbtiles##g' | sed 's#,$##')
    output="./overlay_levels/${lvl}.mbtiles"
    martin-cp $(echo "$input" | tr '\n' ' ') --source "$sources" --auto-bounds calc --encoding identity --max-zoom 20 --min-zoom 16 --mbtiles-type normalized --output-file "${output}"
    rm $(echo "$input" | tr '\n' ' ')
done
