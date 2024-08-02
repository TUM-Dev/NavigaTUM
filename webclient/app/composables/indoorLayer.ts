export const indoorLayers = [
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
      "text-offset": 'eval(prop("placement_offset","default"))',
      text:
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
      "font-size": 'eval(prop(lane_default_width,"default"))',
      "text-color": "white",
      "text-opacity": 1,
      "text-halo-radius": 2,
      "text-halo-color": "blue",
      "text-halo-opacity": 0.3,
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
      "border-color": "#000",
    },
  },
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
      "text-offset": 'eval(prop("placement_offset","default"))',
      text: 'eval(has_tag_key("level")?concat(prop(text)," (",get(split(";",tag("level")),0),")"):prop(text))',
      "font-size": 'eval(prop(lane_default_width,"default"))',
      "text-color": "white",
      "text-opacity": 1,
      "text-halo-radius": 2,
      "text-halo-color": "blue",
      "text-halo-opacity": 0.3,
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
];
