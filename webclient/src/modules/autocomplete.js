navigatum.registerModule("autocomplete", (function() {
    function getVisibleElements() {
        var visible = [];
        for (var i in navigatum.app.search.autocomplete.sections) {
            var s = navigatum.app.search.autocomplete.sections[i];
            for (var j in s.entries) {
                if (s.n_visible === undefined || j < s.n_visible || s.expanded)
                    visible.push(s.entries[j].id);
            }
        }
        return visible;
    }
    
    function extract_facets(data) {
        var sections = [];
        for (var i in data.sections) {
            var entries = [];
            for (var j in data.sections[i].entries) {
                // Search uses DC3 and DC1 to mark the beginning/end 
                // of a highlighted sequence:
                // https://en.wikipedia.org/wiki/C0_and_C1_control_codes#Modified_C0_control_code_sets
                var e = data.sections[i].entries[j];
                var name = new Option(e.name).innerHTML
                                            .replace(/\x19/g, "<em>")
                                            .replace(/\x17/g, "</em>");
                var parsed_id = new Option(e.parsed_id).innerHTML
                                                .replace(/\x19/g, "<em>")
                                                .replace(/\x17/g, "</em>");
                var subtext_bold = new Option(e.subtext_bold).innerHTML
                                                .replace(/\x19/g, "<em>")
                                                .replace(/\x17/g, "</em>");
                entries.push({
                    id: e.id,
                    name: name,
                    type: e.type,
                    subtext: e.subtext,
                    subtext_bold: subtext_bold,
                    parsed_id: parsed_id,
                });
            }
            
            if (data.sections[i].facet == "sites_buildings") {
                sections.push({
                    name: "${{ _.search.sections.buildings }}$",
                    expanded: false,
                    entries: entries,
                    nb_hits: data.sections[i].nb_hits,
                    n_visible: data.sections[i].n_visible,
                });
            } else if (data.sections[i].facet == "rooms") {
                sections.push({
                    name: "${{ _.search.sections.rooms }}$",
                    entries: entries,
                    nb_hits: data.sections[i].nb_hits,
                });
            }
        }
        
        return sections;
    }
    
    // As a simple measure against out-of-order responses
    // to the autocompletion, we count queries and make sure
    // that late results will not overwrite the currently
    // visible results.
    var query_counter = 0;
    var latest_used_query_id = null;
    
    return {
        init: function() {},
        oninput: function(q) {
            navigatum.app.search.autocomplete.highlighted = null;
            
            if (q.length == 0) {
                navigatum.app.search.autocomplete.sections = [];
            } else {
                var query_id = query_counter++;
                
                // no-cache instructs browser, because the cached_fetch will store the reponse.
                cached_fetch.fetch(navigatum.api_base + 'search?q=' + window.encodeURIComponent(q),
                                   {cache: "no-cache"}) 
                    .then(data => {
                        // Data will be cached anyway in case the user hits backspace,
                        // but we need to discard the data here if it arrived out of order.
                        if (!(latest_used_query_id) || query_id > latest_used_query_id) {
                            latest_used_query_id = query_id;
                            
                            var sections = extract_facets(data)
                            navigatum.app.search.autocomplete.sections = sections;
                        }
                    });
            }
        },
        extract_facets: extract_facets,
        onkeydown: function(e) {
            switch (e.keyCode) {
                case 27:  // ESC
                    document.getElementById("search").blur();
                    break;
                
                case 40:  // Arrow down
                    var visible = getVisibleElements();
                    var index = visible.indexOf(navigatum.app.search.autocomplete.highlighted);
                    if (index == -1 && visible.length > 0) {
                        navigatum.app.search.autocomplete.highlighted = visible[0];
                    } else if (index >= 0 && index < visible.length - 1) {
                        navigatum.app.search.autocomplete.highlighted = visible[index+1];
                    }
                    e.preventDefault();
                    break;
                
                case 38: // Arrow up
                    var visible = getVisibleElements();
                    var index = visible.indexOf(navigatum.app.search.autocomplete.highlighted);
                    if (index == 0) {
                        navigatum.app.search.autocomplete.highlighted = null;
                    } else if (index > 0) {
                        navigatum.app.search.autocomplete.highlighted = visible[index-1];
                    }
                    e.preventDefault();
                    break;
                
                case 13: // Enter
                    if (navigatum.app.search.autocomplete.highlighted !== null) {
                        navigatum.app.searchGoTo(navigatum.app.search.autocomplete.highlighted, true);
                    } else {
                        navigatum.app.searchGo(false);
                    }
                    break;
            }
        }
    }
})());
