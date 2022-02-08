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
                el_css.href = "/* @echo app_prefix */css/leaflet-with-plugins.css";
                head.appendChild(el_css);

                // JS should trigger init on load
                var el_js = document.createElement("script");
                el_js.src = "/* @echo app_prefix */js/leaflet-with-plugins.min.js";
                el_js.onload = function() {
                    initLeaflet(_this);
                    resolve();
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
                // create the gl context with MSAA antialiasing, so custom layers are antialiasing.
                // slower, but prettier and therefore worth it for our use case
                antialias: true,

                //preview of the following style is available at
                // https://api.mapbox.com/styles/v1/commanderstorm/ckzdc14en003m14l9l8iqwotq.html?title=copy&access_token=pk.eyJ1IjoiY29tbWFuZGVyc3Rvcm0iLCJhIjoiY2t6ZGJyNDBoMDU2ZzJvcGN2eTg2cWtxaSJ9.PY6Drc3tYHGqSy0UVmVnCg&zoomwheel=true&fresh=true#16.78/48.264624/11.670726
                style: 'mapbox://styles/commanderstorm/ckzdc14en003m14l9l8iqwotq',
                accessToken: 'pk.eyJ1IjoiY29tbWFuZGVyc3Rvcm0iLCJhIjoiY2t6ZGJyNDBoMDU2ZzJvcGN2eTg2cWtxaSJ9.PY6Drc3tYHGqSy0UVmVnCg',

                // to prevent the map from stuttering too much while rendering
                interactive: false,
                attribution: '© <a href="https://www.mapbox.com/about/maps/">Mapbox</a> © <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a> <a href="https://www.mapbox.com/map-feedback/" target="_blank">Improve this map</a>'
            }).addTo(map);
            return map;
        },
        icon: null,
    }
})());
