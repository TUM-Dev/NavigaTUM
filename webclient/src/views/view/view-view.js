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
            image:{
                shown_image: null,
                shown_image_id: null,
                slideshow_open: false,
            },
            map: {
                interactive: {
                    map: null,
                    component: null,
                    marker: null,
                    marker2: null,
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
            // Coordinate picker states
            coord_counter: {
                counter: null,
                to_confirm_delete: false,
            },
        }
    },
    beforeRouteEnter: function(to, from, next) { viewNavigateTo(to, from, next, null) },
    beforeRouteUpdate: function(to, from, next) { viewNavigateTo(to, from, next, this) },
    methods: {
        showImageShowcase:function (i, openSlideshow=true){
            if (this.view_data?.imgs && this.view_data.imgs[i]) {
                this.image.slideshow_open = openSlideshow;
                this.image.shown_image_id = i;
                this.image.shown_image = this.view_data.imgs[i];
            }
        },
        hideImageShowcase:function (){
            this.image.slideshow_open = false;
        },
        // This is called
        // - on initial page load
        // - when the view is loaded for the first time
        // - when the view is navigated to from a different view
        // - when the view is navigated to from the same view, but with a different entry
        loadEntryData: function (data) {
            this.view_data = data;

            this.showImageShowcase(0, false);
            
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
            navigatum.setDescription(this.genDescription(data));

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
        genDescription: function(data) {
            const details_for="${{_.view_view.meta.details_for}}$";
            let description=`${details_for} ${data.type_common_name} ${data.name}`;
            if (data.props.computed){
                description+=":"
                for (const prop of data.props.computed){
                    description+=`\n- ${prop.name}: ${prop.text}`;
                }
            }
            return description;
        },
        // --- Loading components ---
        // When these methods are called, the view has already been mounted,
        // so we can find elements by id.
        loadMap: function() {
            if (navigator.userAgent === "Rendertron") {
                return;
            }
            if (this.state.map.selected === "interactive")
                this.loadInteractiveMap();
            else if (this.state.map.selected === "roomfinder")
                this.loadRoomfinderMap(this.state.map.roomfinder.selected_index);
        },
        addLocationPicker: function() {
            window.feedback.close_form();
            this.state.map.selected = "interactive";
            const coords = this.view_data.coords;
            const marker2 = new mapboxgl.Marker({
                draggable: true,
                color: '#ff0000',
            });
            marker2.setLngLat([coords.lon, coords.lat]).addTo(this.map.interactive.map);
            this.map.interactive.marker2 = marker2;
        },
        _genFeedbackBody:function (currentEdits){
            if (Object.keys(currentEdits).length === 0){
                // For no edits, don't show a badly formatted message
                return "";
            }

            let editStr="";
            for (const [key, value] of Object.entries(currentEdits)) {
                editStr += `"${key}": {coords: {lat: ${value.coords.lat}, lon: ${value.coords.lon}}},\n`
            }

            let actionMsg="${{_.feedback.coordinatepicker.add_coordinate}}$"
            if (Object.keys(currentEdits).length>1){
                actionMsg="${{_.feedback.coordinatepicker.edit_multiple_coordinates}}$"
            }
            else if (this.view_data.coords.accuracy !== "building") {
                actionMsg = "${{_.feedback.coordinatepicker.correct_coordinate}}$"
            }

            return `${actionMsg}:\n` +
                "\`\`\`\n" +
                editStr +
                "\`\`\`";
        },
        _openFeedbackForm: function(addCurrentLocation=false){
            // The feedback form is opened. This may be prefilled with previously corrected coordinates.
            // Maybe get the old coordinates from localstorage
            const currentEdits=navigatum.getLocalStorageWithExpiry("coordinate-feedback",{});

            if (addCurrentLocation){
                // add the current edits to the feedback
                const location=this.map.interactive.marker2.getLngLat();
                currentEdits[this.view_data.id]={coords:{lat: location.lat, lon: location.lng}}
                // save to local storage with ttl of 12h (garbage-collected on next read)
                navigatum.setLocalStorageWithExpiry("coordinate-feedback",currentEdits,12);
            }
            const body=this._genFeedbackBody(currentEdits);

            let subjectMsg="${{_.feedback.coordinatepicker.edit_coordinate_subject}}$"
            if (Object.keys(currentEdits).length>1){
                subjectMsg="${{_.feedback.coordinatepicker.edit_coordinates_subject}}$"
            }
            open_feedback("entry", `[${this.view_data.id}]: ${subjectMsg}`, body);
        },
        confirmLocationPicker: function() {
            this._openFeedbackForm(true);
        document.getElementById("feedback-coordinate-picker-helptext").classList.remove("d-none");
            this.map.interactive.marker2.remove();
            this.map.interactive.marker2 = null;
        },
        cancelLocationPicker: function() {
            this.map.interactive.marker2.remove();
            this.map.interactive.marker2 = null;
        },
        loadInteractiveMap: function(from_ui) {
            var _this = this;
            var from_map = this.state.map.selected;

            this.state.map.selected = "interactive";

            const do_map_update = function() {
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
                    marker = new mapboxgl.Marker({element: c.createMarker()});
                    _this.map.interactive.marker = marker;
                    const coords = _this.view_data.coords;
                    marker.setLngLat([coords.lon, coords.lat]).addTo(map);
                    
                    if (_this.view_data.maps && _this.view_data.maps.overlays) {
                        c.setFloorOverlays(
                            _this.view_data.maps.overlays.available,
                            _this.view_data.maps.overlays.default
                        )
                    } else {
                        c.setFloorOverlays(null)
                    }
                    
                    var default_zooms = {
                        joined_building: 16,
                        building: 17,
                        room: 18,
                    }
                    if (from_map === "interactive"){
                        map.flyTo({
                            center: [coords.lon, coords.lat],
                            zoom: default_zooms[_this.view_data.type] ? default_zooms[_this.view_data.type] : 16,
                            speed: 1,
                            maxDuration: 2000
                        });
                    }
                    else {
                        map.setZoom(16);
                        map.setCenter([coords.lon, coords.lat]);
                    }
                });
            }

            // The map element should be visible when initializing
            if (!document.querySelector("#interactive-map .mapboxgl-canvas"))
                this.$nextTick(do_map_update())
            else
                do_map_update()

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
        entry_feedback: function() {
            const picker = document.getElementById("feedback-coordinate-picker");
            picker.onclick = this.addLocationPicker;
            picker.classList.remove("d-none");
            picker.classList.add("activate-coordinatepicker");

            this._openFeedbackForm();
        },
        deletePendingCoordinates: function() {
            if (this.coord_counter.to_confirm_delete) {
                navigatum.removeLocalStorage("coordinate-feedback");
                this.coord_counter.to_confirm_delete = false;
            } else {
                this.coord_counter.to_confirm_delete = true;
            }
        }
    },
    watch: {
        'state.rooms_overview.filter': function() { this.updateRoomsOverview(); },
    },
    mounted: function () {
        this.is_mounted = true;
        if (navigator.userAgent === "Rendertron") {
            return;
        }

        // Update pending coordinate counter on localStorage changes
        var _this = this;
        const updateCoordinateCounter = function() {
            const coords = navigatum.getLocalStorageWithExpiry("coordinate-feedback", {});
            _this.coord_counter.counter = Object.keys(coords).length;
        }
        window.addEventListener("storage", updateCoordinateCounter);
        updateCoordinateCounter();

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

