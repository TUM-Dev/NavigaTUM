local function without_cover(levels)
    local merged_levels = {}
    -- merge covering
    for _, level_to_add in ipairs(levels) do
        local was_merged = false
        for merged_idx, merged_level in ipairs(merged_levels) do
            if level_to_add.min > merged_level.max or level_to_add.max < merged_level.min then
                -- they don't overlap => continue to next merged_level
                goto continue
            end
            if level_to_add.min >= merged_level.min and level_to_add.max <= merged_level.max then
                -- l1 is fully covered by l2 => no action requred, can be discarded, continue to next level_to_add
                goto next_level
            end
            if level_to_add.min < merged_level.min then
                -- l1 is deeper into min => l2.min should be updated
                merged_levels[merged_idx].min = level_to_add.min
            end
            if level_to_add.max > merged_level.max then
                -- l1 is further into max => l2.max should be updated)
                merged_levels[merged_idx].max = level_to_add.max
            end
            was_merged = true
            ::continue::
        end
        if not was_merged then
            table.insert(merged_levels, level_to_add)
        end
        ::next_level::
    end
    return merged_levels
end

function SantiseLevel(level)
    if level == nil or level == "" then
        return {}
    end
    local de_spaced_level, _ = string.gsub(level, "[ \t]+", "")
    local levels = {}
    -- split via ;
    for lsplit in string.gmatch(de_spaced_level, "([^;]+)") do
        -- unify formating
        local lsplit_sane_dashes, _ = string.gsub(lsplit, "(-?[0-9]+)-(-?[0-9]+)", "%1~%2")
        local lsplit_sane_units, _ = string.gsub(lsplit_sane_dashes, "^(-?[0-9]+)$", "%1~%1")
        --  reverse wrong way
        local range = {}
        range.min = 100
        range.max = -100
        for split_part in string.gmatch(lsplit_sane_units, "([^~]+)") do
            local split_num = tonumber(split_part)
            if split_num == nil then
                -- if this does not parse to a number
                -- it is nonsense and has to be ignored.
                -- Otherwise, this would be just more wrong
                return {}
            end
            range.min = math.min(split_num, range.min)
            range.max = math.max(split_num, range.max)
        end

        table.insert(levels, range)
    end
    return without_cover(levels)
end

return SantiseLevel
