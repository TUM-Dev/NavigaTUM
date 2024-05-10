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

From the root of the repository, run:

```bash
docker run -it -e JAVA_TOOL_OPTIONS="-Xmx10g" -v "$(pwd)/map":/data ghcr.io/onthegomap/planetiler:latest --download --area=bavaria --languages=de,en --Xmx10g --storage=mmap
```

For `planet`, you might want to increase the `--Xmx` parameter to 20GB. For 128GB of RAM or more you will want to
use `--storage=ram` instead of `--storage=mmap`.

### Serve the tileset

After generating `output.mbtiles` you can serve it with a tileserver.
We use [tileserver-gl](https://github.com/maptiler/tileserver-gl) for this, but there are other ones out there.
This may be one optimisation point in the future.

From the root of the repository, run:

```bash
docker compose -f docker-compose.local.yml up --build
```

### Edit the style

For editing the style we use [Maputnik](https://github.com/maputnik/editor).
It is a web-based editor for Maplibre styles.
You can use it to edit the style and see the changes live.

> [!NOTE]
> Maputnik is not fully compatible with tileserver-gl
> Maputnik expects the data on an url, tileserver-gl expects it to be a file.
> For maputnik to accept the file, you need to do the following:

```diff
"openmaptiles": {
  "type": "vector",
-   "url": "mbtiles://output.mbtiles"
+   "url": "http://localhost:7770/data/openmaptiles.json"
},
```

To edit the style you thus need to run maputnik and tileserver-gl at the same time.
Change the style to the version maputnik expects.
You cannot preview the style in tileserver-gl, but you can see the changes in maputnik.

```bash
docker run -it --rm -p 8888:8888 maputnik/editor
```

> [!WARNING]
> After exporting the edited style don't forget to revert the change to the vector url 😉

| Step 1                                                                                         | Step 2                                                                                              |
|------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------|
| ![Where in Maputnik to click to import a style](/resources/documentation/maputnik-import1.png) | ![Where in Maputnik to click then to import a style](/resources/documentation/maputnik-import2.png) |

### Sprites

We get our sprites from [maputnik/osm-liberty](https://github.com/maputnik/osm-liberty) via

```
export TILE_SPRITES_URL=https://raw.githubusercontent.com/maputnik/osm-liberty/gh-pages/sprites
rm sprites/*
wget -P sprites ${TILE_SPRITES_URL}/osm-liberty.json ${TILE_SPRITES_URL}/osm-liberty@2x.json ${TILE_SPRITES_URL}/osm-liberty.png ${TILE_SPRITES_URL}/osm-liberty@2x.png
```
