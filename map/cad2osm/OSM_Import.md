# OSM Import

## ifc to geoparquet
- Use Mybinder environment from https://github.com/map-per/georeference-ifc
- Copy _migrate_ifc_to_geoparquet.ipynb_ and ifc data to Mybinder
- Adjust file name and level in `files`
- Run notebook to generate parquet file

## geoparquet to geojson
- Adjust `filename`
- Run notebook to generate geojson file

## geojson to OSM
- open geojson file with JOSM
- adjust geometry
  - JOSM shortcuts:
    - rotate: Str + Shift + _double click_
    - scale size: Str + Alt + _double click_
    - square areas: q
    - merge node into way: n
    - merge nodes together: m
    - switch to grid view: Str + w
    - (Use advanced JOSM setting `merge-nodes.mode=1` to place new node in the middle of merged nodes) 
- make sure all doors are connected to a room or corridor outline by using the filter (ways:0 AND indoor=door)
- upload to OSM
