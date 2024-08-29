-- See https://github.com/osm2pgsql-dev/osm2pgsql/tree/master/flex-config
-- for configuration examples
SantiseLevel = require(".map.osm2pgsql.levels")

-- For debugging
-- inspect = require('inspect')
-- print(inspect(object))

print("osm2pgsql version: " .. osm2pgsql.version)

local tables = {}
tables.indoor_nodes =
    osm2pgsql.define_node_table(
    "indoor_nodes",
    {
        {column = "tags", type = "jsonb"},
        {column = "geom", type = "point", not_null = true}
    }
)
tables.indoor_ways =
    osm2pgsql.define_way_table(
    "indoor_ways",
    {
        {column = "tags", type = "jsonb"},
        {column = "geom", type = "linestring", not_null = true}
    }
)
tables.indoor_polygons =
    osm2pgsql.define_area_table(
    "indoor_polygons",
    {
        {column = "type", type = "text"},
        {column = "tags", type = "jsonb"},
        -- The type of the `geom` column is `geometry`, because we need to store
        -- polygons AND multipolygons
        {column = "geom", type = "geometry", not_null = true}
    }
)

-- Debug output: Show definition of tables
for name, dtable in pairs(tables) do
    print("\ntable '" .. name .. "':")
    print("  name='" .. dtable:name() .. "'")
    --    print("  columns=" .. inspect(dtable:columns()))
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
    "mml:class"
}

local clean_useless_tags = osm2pgsql.make_clean_tags_func(delete_keys)

-- Helper function to remove some of the tags we usually are not interested in.
-- Returns true if there are no tags left.
local function clean_tags(tags)
    if clean_useless_tags(tags) then
        return true
    end
    -- clean up the indoor tags
    if tags.indoor == nil and tags.level == nil then
        return true
    end
    if tags.level == nil then
        if tags.layer ~= nil then
            -- usually, this is something which is wrongly tagged or if we use the layer, it has the same effect
            tags.level = tags.layer
        else
            tags.level = "0"
        end
    end
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
    tags.inside = nil -- used to infer indoor, but nothing else

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

-- Called for every node in the input. The `object` argument contains all the
-- attributes of the node like `id`, `version`, etc. as well as all tags as a
-- Lua table (`object.tags`).
function osm2pgsql.process_node(object)
    --  Uncomment next line to look at the object data:
    --  print(inspect(object))

    if clean_tags(object.tags) then
        return
    end
    -- pois should not need layers. Using them is likely a bug
    object.tags.layer = nil
    for _, level in ipairs(SantiseLevel(object.tags.level)) do
        object.tags.level = level
        tables.indoor_nodes:insert(
            {
                tags = object.tags,
                geom = object:as_point()
            }
        )
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
    elseif clean_tags(object.tags) then
        return
    end

    for _, level in ipairs(SantiseLevel(object.tags.level)) do
        object.tags.level = level
        -- Very simple check to decide whether a way is a polygon or not, in a
        -- real stylesheet we'd have to also look at the tags...
        if object.is_closed then
            tables.indoor_polygons:insert(
                {
                    type = object.type,
                    tags = object.tags,
                    geom = object:as_polygon()
                }
            )
        else
            tables.indoor_ways:insert(
                {
                    tags = object.tags,
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

    if clean_tags(object.tags) then
        return
    end

    -- Store multipolygons and boundaries as polygons
    if object.tags.type == "multipolygon" or object.tags.type == "boundary" then
        for _, level in ipairs(SantiseLevel(object.tags.level)) do
            object.tags.level = level
            tables.indoor_polygons:insert(
                {
                    type = object.type,
                    tags = object.tags,
                    geom = object:as_multipolygon()
                }
            )
        end
    end
end
