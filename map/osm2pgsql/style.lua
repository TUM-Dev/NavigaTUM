-- See https://github.com/osm2pgsql-dev/osm2pgsql/tree/master/flex-config
-- for configuration examples
SantiseLevel = require(".map.osm2pgsql.levels")

-- For debugging
-- inspect = require('inspect')
-- print(inspect(object))

print("osm2pgsql version: " .. osm2pgsql.version)

local tables = {}
tables.doors =
    osm2pgsql.define_node_table(
      "doors",
      {
        { column = "width_cm",  type = "integer", not_null = true },
        { column = "level_min", type = "real",    not_null = true },
        { column = "level_max", type = "real",    not_null = true },
        { column = "geom",      type = "point",   not_null = true }
      }
    )
tables.indoor_ways =
    osm2pgsql.define_way_table(
      "indoor_ways",
      {
        { column = "level_min", type = "real",       not_null = true },
        { column = "level_max", type = "real",       not_null = true },
        { column = "geom",      type = "linestring", not_null = true }
      }
    )
tables.rooms =
    osm2pgsql.define_area_table(
      "rooms",
      {
        { column = "indoor",               type = "text",     not_null = true },
        { column = "ref_tum",              type = "text" },
        { column = "students_have_access", type = "boolean",  not_null = true },
        -- Mirror the WC attribute flags carried on `pois`, so room-fill styling can dim
        -- non-matching toilet rooms the same way the icon filter hides their POIs.
        { column = "is_male_toilet",       type = "boolean",  not_null = true },
        { column = "is_female_toilet",     type = "boolean",  not_null = true },
        { column = "is_wheelchair_toilet", type = "boolean",  not_null = true },
        { column = "level_min",            type = "real",     not_null = true },
        { column = "level_max",            type = "real",     not_null = true },
        -- The type of the `geom` column is `geometry`, because we need to store
        -- polygons AND multipolygons
        { column = "geom",                 type = "geometry", not_null = true }
      }
    )
tables.pois =
    osm2pgsql.define_table(
      {
        name = "pois",
        -- `any` ids (osm_type + osm_id) so nodes can share this table: the `area` id type a
        -- `define_area_table` gives rejects node inserts, but bare amenity=toilets points are nodes.
        ids = { type = "any", id_column = "osm_id", type_column = "osm_type" },
        columns = {
          { column = "indoor",               type = "text",    not_null = true },
          { column = "ref",                  type = "text" },
          { column = "name",                 type = "text" },
          { column = "students_have_access", type = "boolean", not_null = true },
          { column = "is_male_toilet",       type = "boolean", not_null = true },
          { column = "is_female_toilet",     type = "boolean", not_null = true },
          { column = "is_wheelchair_toilet", type = "boolean", not_null = true },
          { column = "area",                 type = "real",    not_null = true },
          { column = "level_min",            type = "real",    not_null = true },
          { column = "level_max",            type = "real",    not_null = true },
          -- a point is all we render; the geometry is reduced to one before insertion.
          { column = "geom",                 type = "point",   not_null = true }
        }
      }
    )

-- Debug output: Show definition of tables
for name, _ in pairs(tables) do
  print("\ntable '" .. name .. "'")
  -- print("  name='" .. dtable:name() .. "'")
  -- print("  columns=" .. inspect(dtable:columns()))
end

-- These tag keys are generally regarded as useless for most rendering. Most
-- of them are from imports or intended as internal information for mappers.
--
-- If a key ends in '*' it will match all keys with the specified prefix.
--
-- If you want some of these keys, perhaps for a debugging layer, just
-- delete the corresponding lines.
local delete_keys = {
  -- "mapper" keys
  "attribution",
  "comment",
  "created_by",
  "fixme",
  "note",
  "note:*",
  "odbl",
  "odbl:note",
  "source",
  "source:*",
  "source_ref",
  -- "import" keys

  -- Corine Land Cover (CLC) (Europe)
  "CLC:*",
  -- Geobase (CA)
  "geobase:*",
  -- CanVec (CA)
  "canvec:*",
  -- osak (DK)
  "osak:*",
  -- kms (DK)
  "kms:*",
  -- ngbe (ES)
  -- See also note:es and source:file above
  "ngbe:*",
  -- Friuli Venezia Giulia (IT)
  "it:fvg:*",
  -- KSJ2 (JA)
  -- See also note:ja and source_ref above
  "KSJ2:*",
  -- Yahoo/ALPS (JA)
  "yh:*",
  -- LINZ (NZ)
  "LINZ2OSM:*",
  "linz2osm:*",
  "LINZ:*",
  "ref:linz:*",
  -- WroclawGIS (PL)
  "WroclawGIS:*",
  -- Naptan (UK)
  "naptan:*",
  -- TIGER (US)
  "tiger:*",
  -- GNIS (US)
  "gnis:*",
  -- National Hydrography Dataset (US)
  "NHD:*",
  "nhd:*",
  -- mvdgis (Montevideo, UY)
  "mvdgis:*",
  -- EUROSHA (Various countries)
  "project:eurosha_2012",
  -- UrbIS (Brussels, BE)
  "ref:UrbIS",
  -- NHN (CA)
  "accuracy:meters",
  "sub_sea:type",
  "waterway:type",
  -- StatsCan (CA)
  "statscan:rbuid",
  -- RUIAN (CZ)
  "ref:ruian:addr",
  "ref:ruian",
  "building:ruian:type",
  -- DIBAVOD (CZ)
  "dibavod:id",
  -- UIR-ADR (CZ)
  "uir_adr:ADRESA_KOD",
  -- GST (DK)
  "gst:feat_id",
  -- Maa-amet (EE)
  "maaamet:ETAK",
  -- FANTOIR (FR)
  "ref:FR:FANTOIR",
  -- 3dshapes (NL)
  "3dshapes:ggmodelk",
  -- AND (NL)
  "AND_nosr_r",
  -- OPPDATERIN (NO)
  "OPPDATERIN",
  -- Various imports (PL)
  "addr:city:simc",
  "addr:street:sym_ul",
  "building:usage:pl",
  "building:use:pl",
  -- TERYT (PL)
  "teryt:simc",
  -- RABA (SK)
  "raba:id",
  -- DCGIS (Washington DC, US)
  "dcgis:gis_id",
  -- Building Identification Number (New York, US)
  "nycdoitt:bin",
  -- Chicago Building Inport (US)
  "chicago:building_id",
  -- Louisville, Kentucky/Building Outlines Import (US)
  "lojic:bgnum",
  -- MassGIS (Massachusetts, US)
  "massgis:way_id",
  -- Los Angeles County building ID (US)
  "lacounty:*",
  -- Address import from Bundesamt für Eich- und Vermessungswesen (AT)
  "at_bev:addr_date",
  -- misc
  "import",
  "import_uuid",
  "OBJTYPE",
  "SK53_bulk:load",
  "mml:class",
  -- we are not doing 3D
  "height"
}

local clean_useless_tags = osm2pgsql.make_clean_tags_func(delete_keys)

-- Helper function to remove some of the tags we usually are not interested in.
-- Returns true if there are no tags left.
local function clean_tags_indoor(tags)
  if clean_useless_tags(tags) then
    return true
  end
  -- clean up the indoor tags
  -- not relevant for us
  if (tags.indoor == nil and tags.level == nil) or tags.indoor == "no" or tags.indoor == "false" or tags.indoor == "level" then
    return true
  end
  -- clean up levels and layer misconceptions
  if tags.level == nil then
    if tags.layer ~= nil then
      -- usually, this is something which is wrongly tagged or if we use the layer, it has the same effect
      tags.level = tags.layer
    else
      tags.level = "0"
    end
  end
  -- infer the indoor tag
  if tags.indoor == nil then
    -- need to infer indoor tag
    if tags.inside ~= nil then
      tags.indoor = tags.inside
    elseif tags.room ~= nil then
      tags.indoor = "room"
    elseif tags.area ~= nil then
      tags.indoor = "area"
    else
      tags.indoor = "yes"
    end
  end
  tags.inside = nil   -- used to infer indoor, but nothing else
  -- to simplify our data model, we smudge some room= tags into the indoor= space
  if tags.indoor == "room" then
    if tags.room == "toilet" or tags.room == "toilets" or tags.room == "shower" or tags.room == "bathroom" or tags.amenity == "toilet" or tags.amenity == "toilets" then
      tags.indoor = "toilet"
    elseif tags.amenity == "shower" or tags.amenity == "showers" or tags.room == "shower" or tags.room == "showers" or tags.indoor == "shower" or tags.indoor == "showers" then
      tags.indoor = "shower"
    elseif tags.room == "elevator" then
      tags.indoor = "elevator"
    elseif tags.room == "stairs" then
      tags.indoor = "stairs"
    elseif tags.room == "auditorium" or tags.room == "lecture_hall" then
      tags.indoor = "auditorium"
    end
  end
  -- we will never show more than a poi icon here
  if tags.indoor == "elevator" or tags.indoor == "shower" or tags.indoor == "toilet" then
    tags.ref = nil
    tags.name = nil
  end
  -- A stairwell usually spans several levels, so its ref:tum/ref/name are semicolon-joined
  -- multi-values (e.g. "U499D;0499D;1499D") that are meaningless on a single footprint. Render
  -- stairs like a corridor (a plain room fill, no labelled poi) and drop the per-level identifiers.
  if tags.indoor == "stairs" then
    tags.ref = nil
    tags.name = nil
    tags["ref:tum"] = nil
  end

  -- why are there so many objects with just the layer set, nothing else
  if tags.indoor == nil and tags.level ~= nil and #(tags) == 1 then
    return true
  end
  -- why do people like mapping clocks so much??
  -- they are not usefully for us (or likely anybody)
  if tags.amenity == "clock" then
    return true
  end

  return next(tags) == nil
end

-- The typed toilet attribute booleans for a feature's tags. A unisex/all-gender toilet is usable
-- by everyone; with no separate unisex flag it is encoded as both male and female. The flags only
-- apply to `indoor == "toilet"`.
local function toilet_flags(tags)
  local is_toilet = tags.indoor == "toilet"
  local unisex = tags.unisex == "yes"
  return {
    male = is_toilet and (tags.male == "yes" or unisex),
    female = is_toilet and (tags.female == "yes" or unisex),
    wheelchair = is_toilet and tags.wheelchair == "yes",
  }
end

-- Called for every node in the input. The `object` argument contains all the
-- attributes of the node like `id`, `version`, etc. as well as all tags as a
-- Lua table (`object.tags`).
function osm2pgsql.process_node(object)
  --  Uncomment next line to look at the object data:
  --  print(inspect(object))

  if clean_tags_indoor(object.tags) then
    return
  end
  -- Bare amenity=toilets/shower nodes lack a room= tag, so normalise them into the poi space
  -- here (nodes only; ways/relations get smudged in clean_tags_indoor). The indoor-or-level
  -- guard there keeps context-less outdoor amenities out.
  if object.tags.indoor ~= "toilet" and object.tags.indoor ~= "shower" then
    if object.tags.amenity == "toilet" or object.tags.amenity == "toilets" then
      object.tags.indoor = "toilet"
    elseif object.tags.amenity == "shower" or object.tags.amenity == "showers" then
      object.tags.indoor = "shower"
    end
  end
  -- Student-card validators are vending machines in OSM; lift them into their own poi category.
  if object.tags.amenity == "vending_machine" and object.tags.vending == "student_card_validation" then
    object.tags.indoor = "card_validator"
  end
  if object.tags.indoor == "door" then
    -- pois should not need layers. Using them is likely a bug
    object.tags.layer = nil
    -- we want the width_cm, no width_m
    -- invalid or unset widths get 86cm
    if object.tags.width ~= nil then
      object.tags.width = tonumber(object.tags.width)
    end
    if object.tags.width == nil then
      object.tags.width = 86
    else
      object.tags.width = object.tags.width * 100
    end
    for _, level in ipairs(SantiseLevel(object.tags.level)) do
      tables.doors:insert(
        {
          width_cm = object.tags.width,
          level_min = level.min,
          level_max = level.max,
          geom = object:as_point()
        }
      )
    end
  elseif object.tags.indoor == "toilet" or object.tags.indoor == "shower" then
    -- Point geometry has no area, so synthesize `area = 0` (the icon does not use it). No
    -- name/ref: these render as icons only.
    local wc = toilet_flags(object.tags)
    for _, level in ipairs(SantiseLevel(object.tags.level)) do
      tables.pois:insert(
        {
          indoor = object.tags.indoor,
          students_have_access = object.tags.access ~= "private" and object.tags.access ~= "no",
          is_male_toilet = wc.male,
          is_female_toilet = wc.female,
          is_wheelchair_toilet = wc.wheelchair,
          area = 0,
          level_min = level.min,
          level_max = level.max,
          geom = object:as_point()
        }
      )
    end
  elseif object.tags.indoor == "card_validator" then
    -- Unlike toilets, these render as named markers, so carry name + ref:tum into the poi.
    for _, level in ipairs(SantiseLevel(object.tags.level)) do
      tables.pois:insert(
        {
          indoor = object.tags.indoor,
          name = object.tags.name,
          ref = object.tags["ref:tum"],
          students_have_access = true,
          is_male_toilet = false,
          is_female_toilet = false,
          is_unisex_toilet = false,
          is_wheelchair_toilet = false,
          area = 0,
          level_min = level.min,
          level_max = level.max,
          geom = object:as_point()
        }
      )
    end
  elseif object.tags.indoor == "card_validator" then
    -- Unlike toilets, these render as named markers, so carry name + ref:tum into the poi.
    for _, level in ipairs(SantiseLevel(object.tags.level)) do
      tables.pois:insert(
        {
          indoor = object.tags.indoor,
          name = object.tags.name,
          ref = object.tags["ref:tum"],
          students_have_access = true,
          is_male_toilet = false,
          is_female_toilet = false,
          is_unisex_toilet = false,
          is_wheelchair_toilet = false,
          area = 0,
          level_min = level.min,
          level_max = level.max,
          geom = object:as_point()
        }
      )
    end
  end
end

-- Called for every way in the input. The `object` argument contains the same
-- information as with nodes and additionally a boolean `is_closed` flag and
-- the list of node IDs referenced by the way (`object.nodes`).
function osm2pgsql.process_way(object)
  --  Uncomment next line to look at the object data:
  --  print(inspect(object))
  if object.tags.building ~= nil then
    object.tags.indoor = nil
    object.tags.level = nil
    object.tags.inside = nil
  elseif clean_tags_indoor(object.tags) then
    return
  end

  for _, level in ipairs(SantiseLevel(object.tags.level)) do
    object.tags.level_min = level.min
    object.tags.level_max = level.max
    local wc = toilet_flags(object.tags)
    -- Very simple check to decide whether a way is a polygon or not, in a
    -- real stylesheet we'd have to also look at the tags...
    if object.is_closed then
      local geom = object:as_polygon()
      tables.rooms:insert(
        {
          indoor = object.tags.indoor,
          ref_tum = object.tags["ref:tum"],
          students_have_access = object.tags.access ~= "private" and object.tags.access ~= "no",
          is_male_toilet = wc.male,
          is_female_toilet = wc.female,
          is_wheelchair_toilet = wc.wheelchair,
          level_min = level.min,
          level_max = level.max,
          geom = geom
        }
      )
      -- Corridors and stairs are plain fills, not labelled pois.
      if object.tags.indoor ~= "corridor" and object.tags.indoor ~= "stairs" then
        local geom_point = nil
        if object.tags.indoor == "elevator" then
          -- elevators get an icon -> needs no need to strech
          -- looks slightly better if in the middle
          geom_point = geom:centroid()
        else
          geom_point = geom:pole_of_inaccessibility({ stretch = 2 })
        end
        tables.pois:insert(
          {
            indoor = object.tags.indoor,
            name = object.tags.name,
            ref = object.tags.ref,
            students_have_access = object.tags.access ~= "private" and object.tags.access ~= "no",
            is_male_toilet = wc.male,
            is_female_toilet = wc.female,
            is_wheelchair_toilet = wc.wheelchair,
            area = geom:spherical_area(),
            level_min = level.min,
            level_max = level.max,
            geom = geom_point
          }
        )
      end
    else
      tables.indoor_ways:insert(
        {
          level_min = level.min,
          level_max = level.max,
          indoor = object.tags.indoor,
          geom = object:as_linestring()
        }
      )
    end
  end
end

-- Called for every relation in the input. The `object` argument contains the
-- same information as with nodes and additionally an array of members
-- (`object.members`).
function osm2pgsql.process_relation(object)
  --  Uncomment next line to look at the object data:
  --  print(inspect(object))

  if clean_tags_indoor(object.tags) then
    return
  end

  -- Store multipolygons and boundaries as polygons
  if object.tags.type == "multipolygon" or
      object.tags.type == "boundary" then
    local geom = object:as_multipolygon()
    local wc = toilet_flags(object.tags)
    for _, level in ipairs(SantiseLevel(object.tags.level)) do
      tables.rooms:insert(
        {
          indoor = object.tags.indoor,
          ref_tum = object.tags["ref:tum"],
          students_have_access = object.tags.access ~= "private" and object.tags.access ~= "no",
          is_male_toilet = wc.male,
          is_female_toilet = wc.female,
          is_wheelchair_toilet = wc.wheelchair,
          level_min = level.min,
          level_max = level.max,
          geom = geom
        }
      )
      -- Corridors and stairs are plain fills, not labelled pois.
      if object.tags.indoor ~= "corridor" and object.tags.indoor ~= "stairs" then
        -- The pole_of_inaccessibility() function only works for polygons,
        -- not multipolygons. So we split up the multipolygons here and
        -- calculate the pole for each part separately.
        for g in geom:geometries() do
          local geom_point = nil
          if object.tags.indoor == "elevator" then
            -- elevators get an icon -> needs no need to strech
            -- looks slightly better if in the middle
            geom_point = geom:centroid()
          else
            geom_point = geom:pole_of_inaccessibility({ stretch = 2 })
          end
          tables.pois:insert(
            {
              indoor = object.tags.indoor,
              ref = object.tags.ref,
              students_have_access = object.tags.access ~= "private" and object.tags.access ~= "no",
              is_male_toilet = wc.male,
              is_female_toilet = wc.female,
              is_wheelchair_toilet = wc.wheelchair,
              area = g:area(),
              level_min = level.min,
              level_max = level.max,
              geom = geom_point
            }
          )
        end
      end
    end
  end
end
