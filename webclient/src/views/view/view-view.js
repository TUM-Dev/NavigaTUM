function viewNavigateTo(to, from, next, component) {
    navigatum.beforeNavigate(to, from);

    navigatum.getExtendedData(to.params.id)
        .then(data => {
            function finish() {
                if (component) {
                    next();
                    navigatum.afterNavigate(to, from);
                    component.loadEntryData(data);
                } else {
                    next(vm => {
                        navigatum.afterNavigate(to, from);
                        vm.loadEntryData(data);
                    });
                }
            }

            if (data === null) {
                finish()
            } else if (data.type === "root") {
                next("/");
            } else {
                // Redirect to the correct type if necessary. Technically the type information
                // is not required, but it makes nicer URLs.
                var url_type_name = ({
                    campus: "campus",
                    site: "site",
                    area: "site",  // Currently also "site", maybe "group"? TODO
                    building: "building",
                    joined_building: "building",
                    room: "room",
                    virtual_room: "room"
                })[data.type];
                if (url_type_name === undefined) url_type_name = "view";

                if (data !== null && !to.path.slice(1).startsWith(url_type_name)) {
                    next("/" + url_type_name + "/" + to.params.id);
                } else {
                    finish();
                }
            }
        })
}

var _view_default_state = {
    map: {
        // Can also be "roomfinder". "interactive" is default, because
        // it should show a loading indication.
        selected: "interactive",
        roomfinder: {
            selected_id: null,  // Map id
            selected_index: null,  // Index in the 'available' list
            x: -1023 -10,  // Outside in top left corner
            y: -1023 -10,
            width: 400,
            height: 300,
        },
    },
    buildings_overview: {
        expanded: false,
    },
    rooms_overview: {
        expanded: false,
        selected: null,
        filter: "",
    },
};

navigatum.registerView('view', {
    name: 'view-view',
    template: { gulp_inject: 'view-view.inc' },
    data: function() {
        return {
            view_data: null,
            map: {
                interactive: {
                    map: null,
                    component: null,
                },
            },
            sections: {
                rooms_overview: {
                    combined_count: 0,
                    combined_list: [],
                    display_list: [],
                    _filter_index: {
                        selected: null,
                        list: [],
                    },
                    loading: false,
                },
            },
            // State is preserved when navigating in history.
            // May only contain serializable objects!
            state: navigatum.cloneState(_view_default_state),
            copied: false,
        }
    },
    beforeRouteEnter: function(to, from, next) { viewNavigateTo(to, from, next, null) },
    beforeRouteUpdate: function(to, from, next) { viewNavigateTo(to, from, next, this) },
    methods: {
        // This is called
        // - on initial page load
        // - when the view is loaded for the first time
        // - when the view is naviated to from a different view
        // - when the view is naviated to from the same view, but with a different entry
        loadEntryData: function (data) {
            this.view_data = data;
            if (data === null)
                return;

            // --- Maps ---
            if (!navigatum.tryReuseViewState()) {
                // We need to reset state to default here, else it is preserved from the previous page
                navigatum.applyState(navigatum.cloneState(_view_default_state), this.state);

                this.state.map.selected = data.maps.default;
                // Interactive has to be always available, but roomfinder may be unavailable
                if ("roomfinder" in data.maps) {
                    // Find default map
                    for (var i in data.maps.roomfinder.available) {
                        if (data.maps.roomfinder.available[i].id == data.maps.roomfinder.default) {
                            this.state.map.roomfinder.selected_index = i;
                            this.state.map.roomfinder.selected_id = data.maps.roomfinder.available[i].id;
                        }
                    }
                }
            }

            // Maps can only be loaded after first mount because then the elements are
            // created and can be referenced by id.
            if (this.is_mounted)
                this.loadMap();

            // --- Additional data ---
            navigatum.setTitle(data.name);

            // --- Sections ---
            if (this.view_data.sections && this.view_data.sections.rooms_overview) {
                var usages = this.view_data.sections.rooms_overview.usages;
                var combined_list = [];
                for (i in usages) {
                    combined_list.push(...usages[i].children);
                }
                this.sections.rooms_overview.combined_list = combined_list;
                this.sections.rooms_overview.combined_count = combined_list.length;
                this.updateRoomsOverview();
            }
        },
        // --- Loading components ---
        // When these methods are called, the view has already been mounted,
        // so we can find elements by id.
        loadMap: function() {
            if (this.state.map.selected === "interactive")
                this.loadInteractiveMap();
            else if (this.state.map.selected === "roomfinder")
                this.loadRoomfinderMap(this.state.map.roomfinder.selected_index);
        },
        loadInteractiveMap: function(from_ui) {
            var _this = this;
            var from_map = this.state.map.selected;

            this.state.map.selected = "interactive";

            // The map element should be visible when initializing
            this.$nextTick(function () {
                navigatum.getModule("interactive-map").then(function(c) {
                    _this.map.interactive.component = c;

                    let map = _this.map.interactive.map;
                    let marker = _this.map.interactive.marker;
                    // The map might or might not be initialized depending on the type
                    // of navigation.
                    if (document.getElementById("interactive-map")) {
                        if (document.getElementById("interactive-map").classList.contains("mapboxgl-map")) {
                            marker.remove()
                        }
                        else {
                            map = c.initMap('interactive-map');
                            _this.map.interactive.map = map;

                            document.getElementById("interactive-map").classList.remove("loading");
                        }
                    }
                    marker = c.initMarker();
                    _this.map.interactive.marker = marker;
                    const coords = _this.view_data.coords;
                    marker.setLngLat([coords.lon, coords.lat]).addTo(map);
                    // Use 16 as default zoom for now, TODO: Compute
                    if (from_map === "interactive"){
                        map.flyTo({center: [coords.lon, coords.lat], zoom: 16, duration: 5000});
                    }
                    else {
                        map.setZoom(16);
                        map.setCenter([coords.lon, coords.lat]);
                    }
                });
            });

            // To have an animation when the roomfinder is opened some time later,
            // the cursor is set to 'zero' while the interactive map is displayed.
            this.state.map.roomfinder.x = -1023 -10;
            this.state.map.roomfinder.y = -1023 -10;

            if (from_ui) {
                window.scrollTo(0, 0);
            }
        },
        loadRoomfinderMap: function(map_index, from_ui) {
            var map = this.view_data.maps.roomfinder.available[map_index];
            this.state.map.selected = "roomfinder";
            this.state.map.roomfinder.selected_id = map.id;
            this.state.map.roomfinder.selected_index = map_index;

            // Using the #map-container since the bounding rect is still all zero
            // if we switched here from interactive map
            var rect = document.getElementById("map-container").getBoundingClientRect();
            // -1023px, -1023px is top left corner, 16px = 2*8px is element padding
            this.state.map.roomfinder.x = -1023 + (map.x / map.width)  * (rect.width - 16);

            // We cannot use "height" here as it might be still zero before layouting
            // finished, so we use the aspect ratio here.
            this.state.map.roomfinder.y = -1023 + (map.y / map.height) * (rect.width - 16) * (map.height / map.width);

            this.state.map.roomfinder.width = map.width;
            this.state.map.roomfinder.height = map.height;

            if (from_ui) {
                document.getElementById("map-accordion").checked = false;
                /*window.setTimeout(function() {
                    document.getElementById("roomfinder-map-img").scrollIntoView(false);
                }, 50);*/
                window.scrollTo(0, rect.top + this.state.map.roomfinder.y + 1023 - (window.innerHeight / 2));
            }
        },
        updateRoomsOverview: function(set_selected) {
            var state = this.state.rooms_overview;
            var data = this.view_data.sections.rooms_overview;
            var local = this.sections.rooms_overview;

            if (set_selected !== undefined)
                state.selected = set_selected;

            if (state.selected === null) {
                local.display_list = [];
            } else {
                var base_list = state.selected === -1
                                ? local.combined_list : data.usages[state.selected].children;
                if (state.filter === "") {
                    local.display_list = base_list;
                } else {
                    // Update filter index if required
                    if (state.selected !== local._filter_index.selected) {
                        var rooms =  base_list;
                        local._filter_index.list = [];
                        for (var i in rooms) {
                            rooms[i]._lower = rooms[i].name.toLowerCase();
                            local._filter_index.list.push(rooms[i]);
                        }
                        local._filter_index.selected = state.selected;
                    }

                    var filter = state.filter.toLowerCase();
                    var filtered = [];
                    for (var i in local._filter_index.list) {
                        if (local._filter_index.list[i]._lower.indexOf(filter) >= 0)
                            filtered.push(local._filter_index.list[i]);
                    }
                    local.display_list = filtered;
                }
            }

            // If there are a lot of rooms, updating the DOM takes a while.
            // In this case we first reset the list, show a loading indicator and
            // set the long list a short time later (So DOM can update and the indicator
            // is visile).
            if (local.display_list.length > 150) {
                local.loading = true;
                var tmp = local.display_list;
                local.display_list = [];
                // this.$nextTick doesn't work for some reason, the view freezes
                // before the loading indicator is visible.
                window.setTimeout(function () {
                    local.display_list = tmp;
                    local.loading = false;
                }, 20);
            }
        },
        copy_link: function() {
            // c.f. https://stackoverflow.com/a/30810322
            var textArea = document.createElement("textarea");
            textArea.value = window.location.href;

            // Avoid scrolling to bottom
            textArea.style.top = "0";
            textArea.style.left = "0";
            textArea.style.position = "fixed";

            document.body.appendChild(textArea);
            textArea.focus();
            textArea.select();

            try {
                var success = document.execCommand('copy');
                if (success) {
                    var _this = this;
                    _this.copied = true;
                    window.setTimeout(function() { _this.copied = false }, 1000)
                }
            } catch (err) {
                console.error('Failed to copy to clipboard', err);
            }

            document.body.removeChild(textArea);
        },
        entry_feedback: function(id) {
            open_feedback("entry", "[" + id + "]: ");
        },
    },
    watch: {
        'state.rooms_overview.filter': function() { this.updateRoomsOverview(); },
    },
    mounted: function () {
        this.is_mounted = true;
        this.$nextTick(function () {
            // Even though 'mounted' is called there is no guarantee apparently,
            // that it really is mounted now. For this reason we try to poll now.
            // (Not the best solution probably)
            var timeout_ms = 5;
            var _this = this;
            function poll_map() {
                if (document.getElementById("interactive-map") !== null) {
                    _this.loadMap();
                } else {
                    console.log("'mounted' called, but page doesn't appear to be mounted yet. Retrying to load the map in " + timeout_ms + "ms");
                    window.setTimeout(poll_map, timeout_ms);
                    timeout_ms *= 1.5;

                }
            }

            poll_map();
        })
    }
})

