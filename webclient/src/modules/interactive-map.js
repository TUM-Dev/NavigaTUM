navigatum.registerModule("interactive-map", (function() {
    var _map;
    
    return {
        map: undefined,
        init: function() {
            return new Promise(resolve => {
                const head = document.getElementsByTagName("head")[0];
                // Add CSS first (required by Mapbox)
                const el_css = document.createElement("link");
                el_css.rel = "stylesheet";
                el_css.href = "/* @echo app_prefix */css/mapbox/* @if target='release' */.min/* @endif */.css";
                head.appendChild(el_css);

                // JS should trigger init on load
                const el_js = document.createElement("script");
                el_js.src = "/* @echo app_prefix */js/mapbox/* @if target='release' */.min/* @endif */.js";
                el_js.onload = () => {
                    resolve();
                }
                head.appendChild(el_js);
            });
        },
        initMarker: function () {
            const markerDiv = document.createElement('div');
            const markerIcon = document.createElement('span');
            markerIcon.style.backgroundImage = `url(/* @echo app_prefix */assets/map-marker_pin.webp)`;
            markerIcon.style.width = `25px`;
            markerIcon.style.height = `36px`;
            markerIcon.style.top = `-33px`;
            markerIcon.style.left = `-12px`;
            markerIcon.classList.add("marker")
            markerDiv.appendChild(markerIcon);
            const markerShadow = document.createElement('span');
            markerShadow.style.backgroundImage = `url(/* @echo app_prefix */assets/map-marker_pin-shadow.webp)`;
            markerShadow.style.width = `38px`;
            markerShadow.style.height = `24px`;
            markerShadow.style.top = `-20px`;
            markerShadow.style.left = `-12px`;
            markerShadow.classList.add("marker")
            markerDiv.appendChild(markerShadow);
            return new mapboxgl.Marker({element:markerDiv});
        },
        initMap: function(container_id) {
            mapboxgl.accessToken= 'pk.eyJ1IjoiY29tbWFuZGVyc3Rvcm0iLCJhIjoiY2t6ZGJyNDBoMDU2ZzJvcGN2eTg2cWtxaSJ9.PY6Drc3tYHGqSy0UVmVnCg'
            const map= new mapboxgl.Map({
                container: container_id,

                // create the gl context with MSAA antialiasing, so custom layers are antialiasing.
                // slower, but prettier and therefore worth it for our use case
                antialias: true,

                //preview of the following style is available at
                // https://api.mapbox.com/styles/v1/commanderstorm/ckzdc14en003m14l9l8iqwotq.html?title=copy&access_token=pk.eyJ1IjoiY29tbWFuZGVyc3Rvcm0iLCJhIjoiY2t6ZGJyNDBoMDU2ZzJvcGN2eTg2cWtxaSJ9.PY6Drc3tYHGqSy0UVmVnCg&zoomwheel=true&fresh=true#16.78/48.264624/11.670726
                style: 'mapbox://styles/commanderstorm/ckzdc14en003m14l9l8iqwotq?optimize=true',
                
                center: [11.5748, 48.1400],  // Approx Munich
                zoom: 11,  // Zoomed out so that the whole city is visible
            });
            const nav = new mapboxgl.NavigationControl();
            map.addControl(nav, 'top-left');
            
            // Fullscreen currently only on mobile
            if (window.matchMedia &&
                window.matchMedia("only screen and (max-width: 480px)").matches) {
                map.addControl(new mapboxgl.FullscreenControl());
            }
            //const location = new mapboxgl.GeolocateControl({
            //    positionOptions: {
            //    enableHighAccuracy: true
            //    },
            //    trackUserLocation: true,
            //    showUserHeading: true
            //});
            //map.addControl(location);
            
            _map = map;
            
            return map;
        },
        setOverlayImage: function(img_url, coords) {
            // Even if the map is initialized, it could be that
            // it hasn't loaded yet, so we need to postpone adding
            // the overlay layer.
            if (!_map.loaded()) {
                var _this = this;
                _map.on("load", function() {
                    _this.setOverlayImage(img_url, coords);
                });
                return;
            }
            
            if (img_url === null) {  // Remove overlay
                if (_map.getLayer("overlay-layer"))
                    _map.removeLayer("overlay-layer")
                if (_map.getLayer("overlay-bg"))
                    _map.removeLayer("overlay-bg")
                if (_map.getSource("overlay-src"))
                    _map.removeSource("overlay-src")
            } else {
                source = _map.getSource("overlay-src");
                if (!source) {
                    source = _map.addSource("overlay-src", {
                        "type": "image",
                        "url": img_url,
                        "coordinates": coords
                    })
                } else {
                    source.url = img_url;
                    source.coordinates = coors;
                }
                
                layer = _map.getLayer("overlay-layer")
                if (!layer) {
                    _map.addLayer({
                        "id": "overlay-bg",
                        "type": "background",
                        "paint": {
                            "background-color": "#ffffff",
                            "background-opacity": 0.6,
                        }
                    })
                    layer = _map.addLayer({
                        "id": "overlay-layer",
                        "type": "raster",
                        "source": "overlay-src",
                        'paint': {
                            'raster-fade-duration': 0
                        }
                    })
                }
            }
        },
    }
})());
