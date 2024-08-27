-- ececute via
-- lua ./map/osm2pgsql/test.lua

inspect = require("inspect")
SantiseLevel = require(".map.osm2pgsql.levels")

local function test(testname, fn)
    print("--", testname, "--")
    local status, err = pcall(fn)
    if not status then
        print(string.format("error: ./%s", err))
    end
end

---@param o1 any|table First object to compare
---@param o2 any|table Second object to compare
---@param message string True to ignore metatables (a recursive function to tests tables inside tables)
local function equals(o1, o2, message)
    if o1 == o2 then
        return true
    end
    local o1Type = type(o1)
    local o2Type = type(o2)
    if o1Type ~= o2Type then
        return false
    end
    if o1Type ~= "table" then
        return false
    end

    local keySet = {}

    for key1, value1 in pairs(o1) do
        local value2 = o2[key1]
        if value2 == nil or equals(value1, value2, message) == false then
            print(string.format("actual: %s\texpected: %s\tmsg: %s", inspect(o1), inspect(o2), message))
            assert(false)
        end
        keySet[key1] = true
    end

    for key2, _ in pairs(o2) do
        if not keySet[key2] then
            print(string.format("actual: %s\texpected: %s\tmsg: %s", inspect(o1), inspect(o2), message))
            assert(false)
        end
    end
end

test(
    "empty",
    function()
        equals(SantiseLevel(nil), {}, "nil")
        equals(SantiseLevel(""), {}, "empty")
    end
)

test(
    "unit",
    function()
        equals(SantiseLevel("1"), {"1~1"}, "single")
        equals(SantiseLevel(" 1 "), {"1~1"}, "single with spaces")
        equals(SantiseLevel("-1"), {"-1~-1"}, "single negative")
    end
)

test(
    "unions",
    function()
        -- 1;2 is not equivalent to "1~2" due to half-floors
        equals(SantiseLevel("1;2"), {"1~1", "2~2"}, "union")
        equals(SantiseLevel("2;1"), {"2~2", "1~1"}, "union reverse")
        equals(SantiseLevel("-1;-2"), {"-1~-1", "-2~-2"}, "union negative")
        equals(SantiseLevel("-2;-1"), {"-2~-2", "-1~-1"}, "union negative reverse")
        equals(SantiseLevel("-1"), {"-1~-1"}, "union nonvoering")
        equals(SantiseLevel("3;1"), {"3~3", "1~1"}, "union nonvoering")
        equals(SantiseLevel("1;3"), {"1~1", "3~3"}, "union nonvoering reverse")
        equals(SantiseLevel("-3;-1"), {"-3~-3", "-1~-1"}, "union nonvoering negative")
        equals(SantiseLevel("-1;-3"), {"-1~-1", "-3~-3"}, "union nonvoering negative reverse")
        equals(SantiseLevel("-1;1"), {"-1~-1", "1~1"}, "union nonvoering mixed")
        equals(SantiseLevel("1;-1"), {"1~1", "-1~-1"}, "union nonvoering mixed reverse")
        equals(SantiseLevel("1;2;2"), {"1~1", "2~2"}, "duplicate union")
        equals(SantiseLevel("2;1;2"), {"2~2", "1~1"}, "duplicate reversed1 union")
        equals(SantiseLevel("1;2;2"), {"1~1", "2~2"}, "duplicate reversed2 union")
        equals(SantiseLevel(" 1 ; 2 "), {"1~1", "2~2"}, "spaced union")
    end
)

test(
    "ranges",
    function()
        equals(SantiseLevel("1-2"), {"1~2"}, "range")
        equals(SantiseLevel("2-1"), {"1~2"}, "range reverse")
        equals(SantiseLevel("-1--2"), {"-2~-1"}, "range negative")
        equals(SantiseLevel("-2--1"), {"-2~-1"}, "range negative reverse")
        equals(SantiseLevel("-1-1"), {"-1~1"}, "range mixed reverse")
        equals(SantiseLevel("-1-1"), {"-1~1"}, "range mixed")
    end
)

test(
    "union ranges-range",
    function()
        equals(SantiseLevel("1-2;0-3"), {"0~3"}, "completely ocovered")
        equals(SantiseLevel("0-3;1-2"), {"0~3"}, "completely ocovered reversed")
        equals(SantiseLevel("0-1;1-2"), {"0~2"}, "extend min touching")
        equals(SantiseLevel("1-2;0-1"), {"0~2"}, "extend max touching")
        equals(SantiseLevel("0-2;1-4"), {"0~4"}, "extend min covering")
        equals(SantiseLevel("1-4;0-2"), {"0~4"}, "extend max covering")
    end
)
