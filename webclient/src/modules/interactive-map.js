navigatum.registerModule("interactive-map", (function() {
    return {
        init: function() {
            return new Promise(resolve => {
                const head = document.getElementsByTagName("head")[0];
                // Add CSS first (required by Mapbox)
                const el_css = document.createElement("link");
                el_css.rel = "stylesheet";
                el_css.href = "/* @echo app_prefix */css/mapbox.css";
                head.appendChild(el_css);

                // JS should trigger init on load
                const el_js = document.createElement("script");
                el_js.src = "/* @echo app_prefix */js/mapbox.js";
                el_js.onload = () => {
                    resolve();
                }
                head.appendChild(el_js);
            });
        },
        initMarker: function () {
            const markerDiv = document.createElement('div');
            const markerIcon = document.createElement('span');
            markerIcon.style.backgroundImage = `url(/* @echo app_prefix */assets/map-marker_pin.png)`;
            markerIcon.style.width = `25px`;
            markerIcon.style.height = `36px`;
            markerIcon.style.top = `-33px`;
            markerIcon.style.left = `-12px`;
            markerIcon.classList.add("marker")
            markerDiv.appendChild(markerIcon);
            const markerShadow = document.createElement('span');
            markerShadow.style.backgroundImage = `url(/* @echo app_prefix */assets/map-marker_pin-shadow.png)`;
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
                style: 'mapbox://styles/commanderstorm/ckzdc14en003m14l9l8iqwotq',
            });
            const nav = new mapboxgl.NavigationControl();
            map.addControl(nav, 'top-left');
            //const location = new mapboxgl.GeolocateControl({
            //    positionOptions: {
            //    enableHighAccuracy: true
            //    },
            //    trackUserLocation: true,
            //    showUserHeading: true
            //});
            //map.addControl(location);
            return map;
        },
    }
})());
