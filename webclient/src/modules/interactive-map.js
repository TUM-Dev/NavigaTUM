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
                // JS should trigger init on load
                var el_js = document.createElement("script");
                el_js.src = "/* @echo app_prefix */js/leaflet-1.7.1-with-plugins.min.js";
                el_js.onload = function() {
                    initLeaflet(_this);
                    resolve();
                }
                head.appendChild(el_js);
            });
        },
        initMap: function(id) {
            var map = L.map('interactive-map');
            // Gesture handling currently only on mobile
            if (window.matchMedia &&
                window.matchMedia("only screen and (max-width: 480px)").matches) {
                map.gestureHandling.enable();
            }
            console.log(map);
            L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
                attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
            }).addTo(map);
            
            return map;
        },
        icon: null,
    }
})());
