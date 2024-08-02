import type { LayerSpecification } from "maplibre-gl";

export const indoorLayers: LayerSpecification[] = [
  {
    id: "indoor-areas",
    type: "fill",
    source: "indoor",
    filter: ["==", "indoor", "area"],
    paint: {
      "fill-color": "#ff0084",
      "fill-outline-color": "#000",
      "fill-antialias": true, // otherwise the outline is invisible sometimes..
      "fill-opacity": 0.5,
    },
  },
  {
    id: "indoor-corridors",
    type: "fill",
    source: "indoor",
    filter: ["==", "indoor", "corridor"],
    paint: {
      "fill-color": "#8dd1fc",
      "fill-opacity": 0.5,
      "fill-outline-color": "#000",
      "fill-antialias": true, // otherwise the outline is invisible sometimes..
    },
  },
  {
    id: "indoor-rooms",
    type: "fill",
    source: "indoor",
    filter: ["==", "indoor", "room"],
    paint: {
      "fill-color": "#e0e0e0",
      "fill-opacity": 0.5,
      "fill-outline-color": "#000",
      "fill-antialias": true, // otherwise the outline is invisible sometimes..
    },
  },
  {
    filter: ["==", "indoor", "wall"],
    id: "indoor-walls",
    type: "fill",
    source: "indoor",
    paint: {
      "fill-color": "#000",
      "fill-opacity": 0.5,
    },
  },
  {
    id: "indoor-doors",
    type: "fill",
    source: "indoor",
    filter: ["==", "indoor", "door"],
    paint: {
      "fill-color": "#00ffcc",
      "fill-opacity": 0.5,
    },
  },
  {
    id: "indoor-roomnames",
    type: "symbol",
    source: "indoor",
    filter: ["==", "indoor", "area"],
    layout: {
      "text-field":
        "eval(" +
        'has_tag_key("level")' +
        '?concat(prop(text)," (",get(split(";",tag("level")),0),")")' +
        ":prop(text))",
      "text-size": 12,
    },
    paint: {
      "text-opacity": 1,
      "text-color": "white",
      "text-halo-width": 2,
      "text-halo-color": "rgb(48, 112, 179, 0.3)",
    },
  },
  {
    id: "indoor-areanames",
    type: "symbol",
    source: "indoor",
    filter: ["==", "indoor", "area"],
    layout: {
      "text-field":
        "eval(" +
        'has_tag_key("amenity")' +
        '?concat(get(split(";",tag(room)),0)," (",get(split(";",tag("amenity")),0),")")' +
        ':has_tag_key("shop")' +
        '?concat(get(split(";",tag(room)),0)," (",get(split(";",tag("shop")),0),")")' +
        ':has_tag_key("name")' +
        '?concat(get(split(";",tag(room)),0)," (",get(split(";",tag("name")),0),")")' +
        ':has_tag_key("level")' +
        '?concat(get(split(";",tag(room)),0)," (",get(split(";",tag("level")),0),")")' +
        ':get(split(";",tag(room)),0))',
      "text-size": 12,
    },
    paint: {
      "text-opacity": 1,
      "text-color": "white",
      "text-halo-width": 2,
      "text-halo-color": "rgb(48, 112, 179, 0.3)",
    },
  },
];
