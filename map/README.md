# Tileserver-maps

This folder contains the static maps tileserver and vector tiles server for NavigaTUM.

## Getting started

As a basis of generating images it is important to have a tileset (`output.mbtiles`) and a stile.
The style is a JSON file that defines how the map should look like.
The tileset is a sqlite database that contains the map data.
A tileserver takes these two components and produces a variety of formats (png, webp, json, etc.) for the frontend.

### generate your own tileset

Sadly tilesets are really large (`germany` is ~10GB, `planet` ~90GB).
Because of limited badwith and storage space we can't provide a tileset for everyone.
You can generate your own tileset from [OpenStreetMap Data](https://osmdata.openstreetmap.de/)
via [planettiler](https://github.com/onthegomap/planetiler) or other equivalent tools.

From our experience the best way to generate a tileset is to
use [planettiler](https://github.com/onthegomap/planetiler), as their perofrmance is by far (other competitors are not
even close by our tests) the best, and they can work in resourece constreained environments.

From a resource perspective, you need about 2x the size of the tileset as free space on your disk and above 10GB in free
RAM.  
If you really need a tileset and can't meet these requirements, shoot us a message, and we'll see what we can do.  
Generating `europe` takes 3h10m on a modern laptop with 32GB RAM and an SSD. The following commands are optimised for
this.

From the root of the repository, run either (depending on your waiting tolerance and available RAM):

- <details><summary>[fast => ~minutes] Only <b>Germany</b> with approx 64GB of RAM</summary>

  ```bash
  docker run -it -e JAVA_TOOL_OPTIONS="-Xmx54g" -v "$(pwd)/map":/data ghcr.io/onthegomap/planetiler:latest --download --download-threads=10 --download-chunk-size-mb=1000 --fetch-wikidata --languages=de,en --area=germany --Xmx10g  --Xmx54g --nodemap-type=sparsearray --nodemap-storage=ram
  ```

  </details>

- <details><summary>[slower => ~1 hour] Only <b>Germany</b> with lower RAM (click to expand)</summary>

  ```bash
  docker run -it -e JAVA_TOOL_OPTIONS="-Xmx10g" -v "$(pwd)/map":/data ghcr.io/onthegomap/planetiler:latest --download --download-threads=10 --download-chunk-size-mb=1000 --fetch-wikidata --languages=de,en --area=germany --Xmx10g --storage=mmap
  ```

  </details>

- <details><summary>[slow => ~3 hours] <b>Planet</b> with approx 128GB of RAM (click to expand)</summary>

  ```bash
  docker run -it -e JAVA_TOOL_OPTIONS="-Xmx100g" -v "$(pwd)/map":/data ghcr.io/onthegomap/planetiler:latest --download --download-threads=10 --download-chunk-size-mb=1000 --fetch-wikidata --languages=de,en --area=planet --bounds=world --Xmx100g --nodemap-type=sparsearray --nodemap-storage=ram
  ```

  </details>

- <details><summary>[slowest => ~24 hours] <b>Planet</b> with lower amounts of RAM (click to expand)</summary>

  ```bash
  docker run -it -e JAVA_TOOL_OPTIONS="-Xmx25g" -v "$(pwd)/map":/data ghcr.io/onthegomap/planetiler:latest --download --download-threads=10 --download-chunk-size-mb=1000 --fetch-wikidata --languages=de,en --area=planet --bounds=world --Xmx25g --nodemap-type=array --storage=mmap
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

### Edit the style

For editing the style we use [Maputnik](https://github.com/maputnik/editor).
It is a web-based editor for Maplibre styles.
You can use it to edit the style and see the changes live.

To edit the style you thus need to run maputnik and martin at the same time.
Change the style to the version maputnik expects.
You cannot preview the style in martin (see [martin#1120](https://github.com/maplibre/martin/issues/1120)), but you can see the changes in maputnik.

To run maputnik, you can either
- use the [instance hosted on github](https://maputnik.github.io/)
- run the same code via
  ```bash
  docker run -it --rm --pull always -p 8888:8888 maputnik/editor:latest
  ```

| Step 1                                                                                         | Step 2                                                                                              |
|------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------|
| ![Where in Maputnik to click to import a style](/resources/documentation/maputnik-import1.png) | ![Where in Maputnik to click then to import a style](/resources/documentation/maputnik-import2.png) |

### Fonts + Sprites for martin

Due to licencing reasons, we cannot directly include the fonts and sprites we use in the project.
You can download them via

```bash
sh ./martin/setup.sh
```

> [!TIP]
> This is already automatically configured in the docker compose file. No need to do extra work
