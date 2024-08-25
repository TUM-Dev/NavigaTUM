-- See https://github.com/osm2pgsql-dev/osm2pgsql/tree/master/flex-config
-- for configuration examples

-- For debugging
-- inspect = require('inspect')
-- print(inspect(object))

print('osm2pgsql version: ' .. osm2pgsql.version)

local tables = {}


tables.indoor_nodes = osm2pgsql.define_node_table('indoor_nodes', {
    { column = 'tags', type = 'jsonb' },
    { column = 'geom',    type = 'point', not_null = true },
})

tables.indoor_ways = osm2pgsql.define_way_table('indoor_ways', {
    { column = 'tags', type = 'jsonb' },
    { column = 'geom', type = 'linestring', not_null = true },
})

tables.indoor_polygons = osm2pgsql.define_area_table('indoor_polygons', {
    { column = 'type', type = 'text' },
    { column = 'tags', type = 'jsonb' },
    -- The type of the `geom` column is `geometry`, because we need to store
    -- polygons AND multipolygons
    { column = 'geom', type = 'geometry', not_null = true },
})

-- Debug output: Show definition of tables
for name, dtable in pairs(tables) do
    print("\ntable '" .. name .. "':")
    print("  name='" .. dtable:name() .. "'")
--    print("  columns=" .. inspect(dtable:columns()))
end

-- Helper function to remove some of the tags we usually are not interested in.
-- Returns true if there are no tags left.
local function clean_tags(tags)
    tags.odbl = nil
    tags.created_by = nil
    tags.source = nil
    tags['source:ref'] = nil

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

    if object.tags.indoor == nil and object.tags.level == nil then
       return
    end
    if object.tags.level == nil then
        if object.tags.layer ~= nil then
            -- usually, this is something which is wrongly tagged or if we use the layer, it has the same effect
            object.tags.level = object.tags.layer
            object.tags.layer = nil -- pois should not need layers
        else
           object.tags.level = '0'
        end
    end
    if object.tags.indoor == nil then
        -- need to infer indoor tag
        if object.tags.room ~= nil then
            object.tags.indoor = 'room'
        else
            object.tags.indoor = 'yes'
        end
    end

    -- why the fuck are there so many objects with just the layer set, nothing else
    if object.tags.indoor == nil and object.tags.level ~= nil and #(object.tags) == 1 then
        return
    end
    -- why do people like mapping clocks so much??
    if object.tags.amenity == 'clock' then
        return
    end

    tables.indoor_nodes:insert({
        tags = object.tags,
        geom = object:as_point()
    })
end

-- Called for every way in the input. The `object` argument contains the same
-- information as with nodes and additionally a boolean `is_closed` flag and
-- the list of node IDs referenced by the way (`object.nodes`).
function osm2pgsql.process_way(object)
    --  Uncomment next line to look at the object data:
    --  print(inspect(object))

    if clean_tags(object.tags) then
        return
    end

    if object.tags.indoor == nil and object.tags.level == nil then
       return
    end

    -- Very simple check to decide whether a way is a polygon or not, in a
    -- real stylesheet we'd have to also look at the tags...
    if object.is_closed then
        tables.indoor_polygons:insert({
            type = object.type,
            tags = object.tags,
            geom = object:as_polygon()
        })
    else
        tables.indoor_ways:insert({
            tags = object.tags,
            geom = object:as_linestring()
        })
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

    if object.tags.indoor == nil and object.tags.level == nil then
       return
    end

    -- Store multipolygons and boundaries as polygons
    if object.tags.type == 'multipolygon' or
       object.tags.type == 'boundary' then
         tables.indoor_polygons:insert({
            type = object.type,
            tags = object.tags,
            geom = object:as_multipolygon()
        })
    end
end
