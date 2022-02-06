navigatum.registerModule("interactive-map", (function() {
    function initLeaflet(_this) {
        _this.icon = L.icon({
            iconUrl: '/* @echo app_prefix */assets/map-marker_pin.png',
            shadowUrl: '/* @echo app_prefix */assets/map-marker_pin-shadow.png',

            iconSize:     [25, 36],
            shadowSize:   [38, 24],
            iconAnchor:   [12, 33],
            shadowAnchor: [12, 20],
            popupAnchor:  [0, -23]
        });
    }
    
    return {
        init: function() {
            var _this = this;
            return new Promise(resolve => {
                var head  = document.getElementsByTagName("head")[0];
                // Add CSS first (required by Leaflet)
                var el_css  = document.createElement("link");
                el_css.rel = "stylesheet";
                el_css.href = "/* @echo app_prefix */css/leaflet-1.7.1-with-plugins.css";
                head.appendChild(el_css);

                var mb_css  = document.createElement("link");
                mb_css.rel = "stylesheet";
                mb_css.href = "https://api.tiles.mapbox.com/mapbox-gl-js/v1.2.0/mapbox-gl.css";
                head.appendChild(mb_css);

                // JS should trigger init on load
                var el_js = document.createElement("script");
                el_js.src = "/* @echo app_prefix */js/leaflet-1.7.1-with-plugins.min.js";
                el_js.onload = function() {
                    initLeaflet(_this);
                    var mb_js = document.createElement("script");
                    mb_js.src = "https://api.tiles.mapbox.com/mapbox-gl-js/v1.2.0/mapbox-gl.js";
                    mb_js.onload = function() {
                        var mbl_js = document.createElement("script");
                        mbl_js.src = "https://cdnjs.cloudflare.com/ajax/libs/mapbox-gl-leaflet/0.0.15/leaflet-mapbox-gl.js";
                        mbl_js.onload = function() {
                            resolve();
                        }
                        head.appendChild(mbl_js);
                    }
                    head.appendChild(mb_js);
                }
                head.appendChild(el_js);
            });
        },
        initMap: function(id) {
            var map = L.map('interactive-map').setView([38.912753, -77.032194], 15);
            // Gesture handling currently only on mobile
            if (window.matchMedia &&
                window.matchMedia("only screen and (max-width: 480px)").matches) {
                map.gestureHandling.enable();
            }
            console.log(map);
            var gl = L.mapboxGL({
                antialias: true,
                // create the gl context with MSAA antialiasing, so custom layers are antialiasing.
                // slower, but prettier and therefore worth it for our use case
                style: 'https://api.maptiler.com/maps/f37dd045-7438-4fff-9bc2-73fce2ba974f/style.json?key=qMbPTihz6isJJ483G8aF',
                interactive: false, // to prevent the map from stuttering too much while rendering
                attribution: '<a href="https://www.openstreetmap.org/copyright">&copy; OpenStreetMap contributors, CC-BY-SA</a> | <a href="https://www.maptiler.com/copyright/">&copy; MapTiler</a>',
            }).addTo(map);
            
            return map;
        },
        icon: null,
    }
})());
