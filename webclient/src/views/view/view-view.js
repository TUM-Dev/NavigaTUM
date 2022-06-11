function viewNavigateTo(to, from, next, component) {
    navigatum.beforeNavigate(to, from);

    navigatum.getData(to.params.id)
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
                let url_type_name = ({
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

const _view_default_state = {
    map: {
        // Can also be "roomfinder". "interactive" is default, because
        // it should show a loading indication.
        selected: "interactive",
        roomfinder: {
            selected_id: null,  // Map id
            selected_index: null,  // Index in the 'available' list
            x: -1023 - 10,  // Outside in top left corner
            y: -1023 - 10,
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
            coord_picker: {
                // The coordinate picker keeps backups of the subject and body
                // in case someone writes a text and then after that clicks
                // the set coordinate button in the feedback form. If we didn't
                // made a backup then, this would be lost after clicking confirm there.
                backup_id: null,
                subject_backup: null,
                body_backup: null,
                force_reopen: false,
            },
            browser_supports_share: "share" in navigator,
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
            let i;
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
                    for (i in data.maps.roomfinder.available) {
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
                const usages = this.view_data.sections.rooms_overview.usages;
                const combined_list = [];
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
            // If this is called from the feedback form using the edit coordinate
            // button, we temporarily save the current subject and body so it is
            // not lost when being reopened
            if (window.feedback
                && document.getElementById("feedback-modal").classList.contains("active")) {
                this.coord_picker.backup_id = this.view_data.id;
                this.coord_picker.subject_backup = document.getElementById("feedback-subject").value;
                this.coord_picker.body_backup = document.getElementById("feedback-body").value;
                this.coord_picker.force_reopen = true;  // reopen after confirm

                window.feedback.closeForm();
            }

            this.state.map.selected = "interactive";

            // Verify that there isn't already a marker (could happen if you click 'assign
            // a location' multiple times from the 'missing accurate location' toast)
            if (this.map.interactive.marker2 === null) {
                // Coordinates are either taken from the entry, or if there are already
                // some in the localStorage use them
                const current_edits = navigatum.getLocalStorageWithExpiry("coordinate-feedback", {});

                const coords = (current_edits[this.view_data.id] || this.view_data).coords;
                const marker2 = new mapboxgl.Marker({
                    draggable: true,
                    color: '#ff0000',
                });
                marker2.setLngLat([coords.lon, coords.lat]).addTo(this.map.interactive.map);
                this.map.interactive.marker2 = marker2;
            }
        },
        _getFeedbackSubject: function(current_edits) {
            if (Object.keys(current_edits).length > 1) {
                return `[${this.view_data.id} et.al.]: ` +
                       "${{_.feedback.coordinatepicker.edit_coordinates_subject}}$";
            }

            let subject_prefix = `[${this.view_data.id}]: `
            let subject_msg = Object.keys(current_edits).length === 0
                              ? "" : "${{_.feedback.coordinatepicker.edit_coordinate_subject}}$";

            // The subject backup is only loaded (and supported) when a single
            // entry is being edited
            if (this.coord_picker.subject_backup
                && this.coord_picker.backup_id === this.view_data.id
                && this.coord_picker.subject_backup !== subject_prefix) {
                const backup = this.coord_picker.subject_backup;
                this.coord_picker.subject_backup = null;
                return backup;
            } else {
                return subject_prefix + subject_msg;
            }
        },
        _getFeedbackBody:function (current_edits) {
            // Look up whether there is a backup of the body and extract the section
            // that is not the coordinate
            let action_msg = "";
            if (this.coord_picker.body_backup
                && this.coord_picker.backup_id === this.view_data.id) {
                let parts = this.coord_picker.body_backup.split("\n\`\`\`");
                if (parts.length === 1) {
                    action_msg = parts[0];
                } else {
                    action_msg = parts[0] + parts[1].split("\`\`\`").slice(1).join("\n");
                }

                this.coord_picker.body_backup = null;
            }

            if (Object.keys(current_edits).length === 0) {
                // For no edits, don't show a badly formatted message
                // (This is "" if there was no backup)
                return action_msg;
            }

            default_action_msg = this.view_data.coords.accuracy === "building"
                                 ? "${{_.feedback.coordinatepicker.add_coordinate}}$"
                                 : "${{_.feedback.coordinatepicker.correct_coordinate}}$";
            action_msg = action_msg || default_action_msg;

            if (Object.keys(current_edits).length > 1){
                // The body backup is discarded if more than a single entry
                // is being edited (because then it is not supported).
                action_msg = "${{_.feedback.coordinatepicker.edit_multiple_coordinates}}$";
            }

            let edit_str="";
            for (const [key, value] of Object.entries(current_edits)) {
                edit_str += `"${key}": {coords: {lat: ${value.coords.lat}, lon: ${value.coords.lon}}},\n`
            }

            return `${action_msg}\n` +
                "\`\`\`\n" +
                edit_str +
                "\`\`\`";
        },
        openFeedbackForm: function(){
            // The feedback form is opened. This may be prefilled with previously corrected coordinates.
            // Maybe get the old coordinates from localstorage
            const current_edits = navigatum.getLocalStorageWithExpiry("coordinate-feedback", {});
            const body = this._getFeedbackBody(current_edits);
            const subject = this._getFeedbackSubject(current_edits);

            document.getElementById("feedback-coordinate-picker")
                    .addEventListener('click', this.addLocationPicker);

            openFeedback("entry", subject, body);
        },
        confirmLocationPicker: function() {
            // add the current edits to the feedback
            const current_edits = navigatum.getLocalStorageWithExpiry("coordinate-feedback", {});
            const location = this.map.interactive.marker2.getLngLat();
            current_edits[this.view_data.id] = {coords: {lat: location.lat, lon: location.lng}}
            // save to local storage with ttl of 12h (garbage-collected on next read)
            navigatum.setLocalStorageWithExpiry("coordinate-feedback", current_edits, 12);

            this.map.interactive.marker2.remove();
            this.map.interactive.marker2 = null;

            // A feedback form is only opened when this is the only (and therefore
            // first coordinate). If there are more coordinates we can assume
            // someone is doing batch edits. They can then use the send button in
            // the coordinate counter at the top of the page.
            if (Object.keys(current_edits).length === 1 || this.coord_picker.force_reopen) {
                this.coord_picker.force_reopen = false;
                this.openFeedbackForm();
            }

            // The helptext (which says thet you can edit multiple coordinates in bulk)
            // is also only shown if there is one edit.
            if (Object.keys(current_edits).length === 1) {
                document.getElementById("feedback-coordinate-picker-helptext")
                        .classList.remove("d-none");
            }
        },
        cancelLocationPicker: function() {
            this.map.interactive.marker2.remove();
            this.map.interactive.marker2 = null;

            if (this.coord_picker.force_reopen) {
                this.coord_picker.force_reopen = false;
                this.openFeedbackForm();
            }
        },
        deletePendingCoordinates: function() {
            if (this.coord_counter.to_confirm_delete) {
                navigatum.removeLocalStorage("coordinate-feedback");
                this.coord_counter.to_confirm_delete = false;
                this.coord_picker.body_backup = null;
                this.coord_picker.subject_backup = null;
                this.coord_picker.backup_id = null;
            } else {
                this.coord_counter.to_confirm_delete = true;
            }
        },
        loadInteractiveMap: function(from_ui) {
            const _this = this;
            const from_map = this.state.map.selected;

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

                    const default_zooms = {
                        joined_building: 16,
                        building: 17,
                        room: 18,
                    };
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
            const map = this.view_data.maps.roomfinder.available[map_index];
            this.state.map.selected = "roomfinder";
            this.state.map.roomfinder.selected_id = map.id;
            this.state.map.roomfinder.selected_index = map_index;

            // Using the #map-container since the bounding rect is still all zero
            // if we switched here from interactive map
            const rect = document.getElementById("map-container").getBoundingClientRect();
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
            let i;
            const state = this.state.rooms_overview;
            const data = this.view_data.sections.rooms_overview;
            const local = this.sections.rooms_overview;

            if (set_selected !== undefined)
                state.selected = set_selected;

            if (state.selected === null) {
                local.display_list = [];
            } else {
                const base_list = state.selected === -1
                    ? local.combined_list : data.usages[state.selected].children;
                if (state.filter === "") {
                    local.display_list = base_list;
                } else {
                    // Update filter index if required
                    if (state.selected !== local._filter_index.selected) {
                        const rooms = base_list;
                        local._filter_index.list = [];
                        for (i in rooms) {
                            rooms[i]._lower = rooms[i].name.toLowerCase();
                            local._filter_index.list.push(rooms[i]);
                        }
                        local._filter_index.selected = state.selected;
                    }

                    const filter = state.filter.toLowerCase();
                    const filtered = [];
                    for (i in local._filter_index.list) {
                        if (local._filter_index.list[i]._lower.indexOf(filter) >= 0)
                            filtered.push(local._filter_index.list[i]);
                    }
                    local.display_list = filtered;
                }
            }

            // If there are a lot of rooms, updating the DOM takes a while.
            // In this case we first reset the list, show a loading indicator and
            // set the long list a short time later (So DOM can update and the indicator
            // is visible).
            if (local.display_list.length > 150) {
                local.loading = true;
                const tmp = local.display_list;
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
            const textArea = document.createElement("textarea");
            textArea.value = window.location.href;

            // Avoid scrolling to bottom
            textArea.style.top = "0";
            textArea.style.left = "0";
            textArea.style.position = "fixed";

            document.body.appendChild(textArea);
            textArea.focus();
            textArea.select();

            try {
                const success = document.execCommand('copy');
                if (success) {
                    const _this = this;
                    _this.copied = true;
                    window.setTimeout(function() { _this.copied = false }, 1000)
                }
            } catch (err) {
                console.error('Failed to copy to clipboard', err);
            }

            document.body.removeChild(textArea);
        },
        share_link: function() {
            if (navigator.share) {
                navigator.share({
                    title: this.view_data.name,
                    text: document.title,
                    url: window.location.href,
                });
            }
        },
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
        const _this = this;
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
            let timeout_ms = 5;
            const _this = this;

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
