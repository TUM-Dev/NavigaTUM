# Tileserver-maps

This folder contains the configuration of how vector tiles server for NavigaTUM.

## Getting started

As a basis of generating images it is important to have a tileset (`output.mbtiles`) and a style:
- The style is a JSON file that defines how the map should look like.
- The tileset is a sqlite database that contains the map data.

A tileserver takes these two components and produces a variety of
formats ([MVT](https://github.com/mapbox/vector-tile-spec), png, webp, json, etc.) for the frontend.

### Edit the style

You cannot currently preview the style in our tileserver martin
(see [martin#1120](https://github.com/maplibre/martin/issues/1120)).
Therefore, for editing the style we use [Maputnik](https://github.com/maputnik/editor).
It is a web-based editor for Maplibre styles.
You can use it to edit the style and see the changes live.

To run maputnik, you can either

- use the [instance hosted on github](https://maputnik.github.io/)
- as an alternative, you can run
  ```bash
  docker run -it --rm --pull always -p 8888:8888 maputnik/editor:latest
  ```

Our style can be found here and can either be "Load[ed] from Url" or uploaded into maputnik manually:

```
https://raw.githubusercontent.com/TUM-Dev/NavigaTUM/refs/heads/main/map/martin/navigatum-basemap.json
```

| Step 1                                                                                         | Step 2                                                                                              |
|------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------|
| ![Where in Maputnik to click to import a style](/resources/documentation/maputnik-import1.png) | ![Where in Maputnik to click then to import a style](/resources/documentation/maputnik-import2.png) |

### generate your own tiles

Sadly tilesets are really large (`germany` is ~10GB, `planet` ~90GB).
Because of limited bandwidth and storage space we can't provide a tileset for everyone.
You can generate your own tileset from [OpenStreetMap Data](https://osmdata.openstreetmap.de/)
via [planetiler](https://github.com/onthegomap/planetiler) or other equivalent tools.

From our experience the best way to generate a tileset is to
use [planetiler](https://github.com/onthegomap/planetiler), as their performance is by far (other competitors are not
even close by our tests) the best, and they can work in resource constrained environments.

From a resource perspective, you need about 2x the size of the tileset as free space on your disk and above 10GB in free
RAM.  
If you really need a tileset and can't meet these requirements, shoot us a message, and we'll see what we can do.  
Generating `europe` takes 3h10m on a modern laptop with 32GB RAM and an SSD. The following commands are optimised for
this.

> [!NOTE]
> below commands expect to be run from the root of the repository

From the root of the repository, run either (depending on your waiting tolerance and available RAM):

- <details><summary>[fast => ~minutes] Only <b>Germany</b> with approx 64GB of RAM</summary>

  ```bash
  docker run --rm --user=$UID -it --pull always \
  -e JAVA_TOOL_OPTIONS="-Xmx54g" -v "$(pwd)/map":/data \
  ghcr.io/onthegomap/planetiler:latest \
  /data/planetiler/shortbread_custom.yml \
  --download --download-threads=10 --download-chunk-size-mb=1000 \
  --free_natural_earth_after_read=true --free_water_polygons_after_read=true --free_lake_centerlines_after_read=true --compress_temp=true \
  --fetch-wikidata --languages=de,en \
  --Xmx54g --nodemap-type=sparsearray --nodemap-storage=ram \
   --area=germany \
   --output=/data/output.mbtiles
  ```

  </details>

- <details><summary>[slower => ~1 hour] Only <b>Germany</b> with lower RAM (click to expand)</summary>

  ```bash
  docker run --rm --user=$UID -it --pull always \
  -e JAVA_TOOL_OPTIONS="-Xmx10g" -v "$(pwd)/map":/data \
  ghcr.io/onthegomap/planetiler:latest \
  /data/planetiler/shortbread_custom.yml \
  --download --download-threads=10 --download-chunk-size-mb=1000 \
  --free_natural_earth_after_read=true --free_water_polygons_after_read=true --free_lake_centerlines_after_read=true --compress_temp=true \
  --fetch-wikidata --languages=de,en \
  --Xmx10g --storage=mmap \
   --area=germany \
   --output=/data/output.mbtiles
  ```

  </details>

- <details><summary>[slow => ~3 hours] <b>Planet</b> with approx 128GB of RAM (click to expand)</summary>

  ```bash
  docker run --rm --user=$UID -it --pull always \
  -e JAVA_TOOL_OPTIONS="-Xmx100g" -v "$(pwd)/map":/data \
  ghcr.io/onthegomap/planetiler:latest \
  /data/planetiler/shortbread_custom.yml \
  --download --download-threads=10 --download-chunk-size-mb=1000 \
  --free_natural_earth_after_read=true --free_water_polygons_after_read=true --free_lake_centerlines_after_read=true --compress_temp=true \
  --fetch-wikidata --languages=de,en \
  --Xmx100g --nodemap-type=sparsearray --nodemap-storage=ram \
  --area=planet --bounds=world \
  --output=/data/output.mbtiles
  ```

  </details>

- <details><summary>[slowest => ~24 hours] <b>Planet</b> with lower amounts of RAM (click to expand)</summary>

  ```bash
  docker run --rm --user=$UID -it --pull always \
  -e JAVA_TOOL_OPTIONS="-Xmx25g" -v "$(pwd)/map":/data \
  ghcr.io/onthegomap/planetiler:latest \
  /data/planetiler/shortbread_custom.yml \
  --download --download-threads=10 --download-chunk-size-mb=1000 \
  --free_natural_earth_after_read=true --free_water_polygons_after_read=true --free_lake_centerlines_after_read=true --compress_temp=true \
  --fetch-wikidata --languages=de,en \
  --Xmx25g --nodemap-type=array --storage=mmap \
  --area=planet --bounds=world \
  --output=/data/output.mbtiles
  ```

  </details>

### Serve the tileset

After generating `output.mbtiles` you can serve it with a tileserver.
We use [martin](https://github.com/maplibre/martin) for this, but there are other ones out there.
This may be one optimisation point in the future.

From the root of the repository, run:

```bash
docker compose -f docker-compose.local.yml up --build
```

> [!TIP]
> For developing which data lands in the style, it can be helpful to run martin locally:
> ```bash
> docker run -p 3000:3000 --rm --user=$UID -it -v "$(pwd)/map":/data \
> ghcr.io/maplibre/martin:latest \
> /data/output.mbtiles
> ```

### Fonts + Sprites for martin

Due to licencing reasons, we cannot directly include the fonts and sprites we use in the project.
You can download them via

```bash
sh ./martin/setup.sh
```

> [!TIP]
> This is already automatically configured in the docker compose file. No need to do extra work

### Adding additional data

If you want to add additional data to the tileset, you can do so by adding a new layer to the style or by modifying an existing one.

You can find more information on how to do this in the [Planetiler documentation](https://github.com/onthegomap/planetiler/tree/main/planetiler-custommap).

To run tests, we recommend downloading the jar file from the [Planetiler releases page](https://github.com/onthegomap/planetiler/releases) and then running the following command:

```bash
java -jar planetiler.jar verify ./map/planetiler/shortbread_custom.yml --watch
```
