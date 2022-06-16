navigatum.registerModule(
  "interactive-map",
  (function () {
    /* global mapboxgl */
    let _map;

    function FloorControl() {}

    // Because mapboxgl might not be loaded yet, we need to postpone
    // the declaration of the FloorControl class
    function floorControlInit() {
      // Add Evented functionality from mapboxgl
      FloorControl.prototype = Object.create(mapboxgl.Evented.prototype);

      FloorControl.prototype.onAdd = function (map) {
        this.map = map;
        this.container = document.createElement("div");
        this.container.classList.add("mapboxgl-ctrl-group");
        this.container.classList.add("mapboxgl-ctrl");
        this.container.classList.add("floor-ctrl");

        // vertical open/collapse button
        const verticalOpenClose = document.createElement("button");
        verticalOpenClose.classList.add("vertical-oc");
        verticalOpenClose.innerHTML =
          "<span id='vertical-oc-text'></span><span class='arrow'>▲</span>";
        verticalOpenClose.addEventListener("click", () => {
          this.container.classList.toggle("closed");
        });
        // horizontal (primarily on mobile)
        const horizontalOpenClose = document.createElement("button");
        horizontalOpenClose.classList.add("horizontal-oc");
        horizontalOpenClose.innerHTML =
          "<span id='horizontal-oc-text'></span><span class='arrow'>❯</span>";
        horizontalOpenClose.addEventListener("click", () => {
          this.container.classList.toggle("closed");
        });

        this.floor_list = document.createElement("div");
        this.floor_list.id = "floor-list";

        this.container.appendChild(horizontalOpenClose);
        this.container.appendChild(this.floor_list);
        this.container.appendChild(verticalOpenClose);

        // To change on `fullscreen` click on mobile, we need to
        // observe window size changed
        if (ResizeObserver) {
          this.resize_observer = new ResizeObserver(() => {
            this._recalculateLayout(this.floor_list.children.length);
          });
          this.resize_observer.observe(
            document.getElementById("interactive-map")
          );
        }

        return this.container;
      };
      FloorControl.prototype.onRemove = function () {
        this.container.parentNode.removeChild(this.container);
        this.map = undefined;
      };
      FloorControl.prototype.updateFloors = function (floors, visibleId) {
        // `floors` is null or a list of floors with data,
        // `visibleId` is the id of the visible floor.
        if (floors === null) {
          this.container.classList.remove("visible");
          this.fire("floor-changed", { file: null, coords: null });
        } else {
          this.floor_list.innerHTML = "";

          const _this = this;
          const clickHandlerBuilder = function (allFloors, i) {
            // Because JS
            return function () {
              if (allFloors) {
                _this._setActiveFloor(i, allFloors[i].floor);
                _this.fire("floor-changed", {
                  file: allFloors[i].file,
                  coords: allFloors[i].coordinates,
                });
              } else {
                _this._setActiveFloor(i, "∅");
                _this.fire("floor-changed", { file: null, coords: null });
              }

              if (!_this.container.classList.contains("reduced"))
                _this.container.classList.add("closed");
            };
          };
          let btn;
          let visibleI = null;
          floors.reverse().forEach((floor, index) => {
            btn = document.createElement("button");
            btn.innerText = floor.floor;
            btn.addEventListener("click", clickHandlerBuilder(floors, index));
            this.floor_list.appendChild(btn);

            if (floor.id === visibleId) visibleI = index;
          });

          if (visibleI === null) {
            this._setActiveFloor(this.floor_list.children.length, "∅");
            this.fire("floor-changed", { file: null, coords: null });
          } else {
            this._setActiveFloor(visibleI, floors[visibleI].floor);
            this.fire("floor-changed", {
              file: floors[visibleI].file,
              coords: floors[visibleI].coordinates,
            });
          }

          // The last button hides all overlays
          btn = document.createElement("button");
          btn.innerText = "∅";
          btn.addEventListener(
            "click",
            clickHandlerBuilder(null, this.floor_list.children.length)
          );
          this.floor_list.appendChild(btn);

          this._recalculateLayout(this.floor_list.children.length);

          this.container.classList.add("visible");
        }
      };
      // Recalculate the layout for displaying n floor buttons
      FloorControl.prototype._recalculateLayout = function (n) {
        // Calculate required and available size to choose between
        // vertical (default) or horizontal layout
        const mapHeight =
          document.getElementById("interactive-map").clientHeight;
        const topCtrlHeight = document.querySelector(
          ".mapboxgl-ctrl-top-left"
        ).clientHeight;
        const bottomCtrlHeight = document.querySelector(
          ".mapboxgl-ctrl-bottom-left"
        ).clientHeight;
        const floorCtrlHeight =
          document.querySelector(".floor-ctrl").clientHeight;

        // The buttons have a height of 29px
        const availableHeight =
          mapHeight - topCtrlHeight - bottomCtrlHeight + floorCtrlHeight;
        const requiredHeight = 29 * n;

        // 3 or less buttons can always be displayed in reduced layout.
        // Also, if the control takes only a small amount of space, it is always open.
        if (n <= 3 || requiredHeight < availableHeight * 0.2) {
          this.container.classList.remove("closed"); // reduced can never be closed
          this.container.classList.remove("horizontal");
          this.container.classList.add("reduced");
        } else {
          this.container.classList.remove("reduced");
          this.container.classList.add("closed");

          // 25px = 10px reserved for top/bottom margin + 5px between control groups
          // 29px = additional height from the open/collapse button
          if (availableHeight - (requiredHeight + 29) > 25)
            this.container.classList.remove("horizontal");
          else this.container.classList.add("horizontal");
        }
      };
      FloorControl.prototype._setActiveFloor = function (floorListI, name) {
        for (let i = 0; i < this.floor_list.children.length; i++) {
          if (i === floorListI)
            this.floor_list.children[i].classList.add("active");
          else this.floor_list.children[i].classList.remove("active");
        }
        document.getElementById("vertical-oc-text").innerText = name;
        document.getElementById("horizontal-oc-text").innerText = name;
      };
    }

    return {
      map: undefined,
      init: function () {
        return new Promise((resolve) => {
          const head = document.getElementsByTagName("head")[0];
          // Add CSS first (required by Mapbox)
          const elCSS = document.createElement("link");
          elCSS.rel = "stylesheet";
          elCSS.href =
            "/* @echo app_prefix */css/mapbox/* @if target='release' */.min/* @endif */.css";
          head.appendChild(elCSS);

          // JS should trigger init on load
          const elJS = document.createElement("script");
          elJS.src =
            "/* @echo app_prefix */js/mapbox/* @if target='release' */.min/* @endif */.js";
          elJS.onload = () => {
            floorControlInit();
            resolve();
          };
          head.appendChild(elJS);
        });
      },
      createMarker: function (hueRotation = 0) {
        const markerDiv = document.createElement("div");
        const markerIcon = document.createElement("span");
        markerIcon.style.backgroundImage = `url(/* @echo app_prefix */assets/map-marker_pin.webp)`;
        markerIcon.style.width = `25px`;
        markerIcon.style.height = `36px`;
        markerIcon.style.filter = `hue-rotate(${hueRotation}deg)`;
        markerIcon.style.top = `-33px`;
        markerIcon.style.left = `-12px`;
        markerIcon.classList.add("marker");
        markerDiv.appendChild(markerIcon);
        const markerShadow = document.createElement("span");
        markerShadow.style.backgroundImage = `url(/* @echo app_prefix */assets/map-marker_pin-shadow.webp)`;
        markerShadow.style.width = `38px`;
        markerShadow.style.height = `24px`;
        markerShadow.style.top = `-20px`;
        markerShadow.style.left = `-12px`;
        markerShadow.classList.add("marker");
        markerDiv.appendChild(markerShadow);
        return markerDiv;
      },
      initMap: function (containerId) {
        mapboxgl.accessToken =
          "pk.eyJ1IjoiY29tbWFuZGVyc3Rvcm0iLCJhIjoiY2t6ZGJyNDBoMDU2ZzJvcGN2eTg2cWtxaSJ9.PY6Drc3tYHGqSy0UVmVnCg";
        const map = new mapboxgl.Map({
          container: containerId,

          // create the gl context with MSAA antialiasing, so custom layers are antialiasing.
          // slower, but prettier and therefore worth it for our use case
          antialias: true,

          // preview of the following style is available at
          // https://api.mapbox.com/styles/v1/commanderstorm/ckzdc14en003m14l9l8iqwotq.html?title=copy&access_token=pk.eyJ1IjoiY29tbWFuZGVyc3Rvcm0iLCJhIjoiY2t6ZGJyNDBoMDU2ZzJvcGN2eTg2cWtxaSJ9.PY6Drc3tYHGqSy0UVmVnCg&zoomwheel=true&fresh=true#16.78/48.264624/11.670726
          style:
            "mapbox://styles/commanderstorm/ckzdc14en003m14l9l8iqwotq?optimize=true",

          center: [11.5748, 48.14], // Approx Munich
          zoom: 11, // Zoomed out so that the whole city is visible

          logoPosition: "bottom-left",
        });
        const nav = new mapboxgl.NavigationControl();
        map.addControl(nav, "top-left");

        // (Browser) Fullscreen is enabled only on mobile, on desktop the map
        // is maximized instead. This is determined once to select the correct
        // container to maximize, and then remains unchanged even if the browser
        // is resized (not relevant for users but for developers).
        const isMobile =
          window.matchMedia &&
          window.matchMedia("only screen and (max-width: 480px)").matches;

        const fullscreenCtl = new mapboxgl.FullscreenControl({
          container: isMobile
            ? document.getElementById("interactive-map")
            : document.getElementById("interactive-map-container"),
        });
        // "Backup" the mapboxgl default fullscreen handler
        fullscreenCtl._onClickFullscreenDefault =
          fullscreenCtl._onClickFullscreen;
        fullscreenCtl._onClickFullscreen = function () {
          if (isMobile) {
            fullscreenCtl._onClickFullscreenDefault();
          } else {
            if (fullscreenCtl._container.classList.contains("maximize")) {
              fullscreenCtl._container.classList.remove("maximize");
              document.body.classList.remove("no-scroll");
            } else {
              fullscreenCtl._container.classList.add("maximize");
              document.body.classList.add("no-scroll");
              // "instant" is not part of the spec but nonetheless implemented
              // by Firefox and Chrome
              window.scrollTo({ top: 0, behavior: "instant" });
            }

            fullscreenCtl._fullscreen =
              fullscreenCtl._container.classList.contains("maximize");
            fullscreenCtl._changeIcon();
            fullscreenCtl._map.resize();
          }
        };
        map.addControl(fullscreenCtl);

        const location = new mapboxgl.GeolocateControl({
          positionOptions: {
            enableHighAccuracy: true,
          },
          trackUserLocation: true,
          showUserHeading: true,
        });
        map.addControl(location);

        // Each source / style change causes the map to get
        // into "loading" state, so map.loaded() is not reliable
        // enough to know whether just the initial loading has
        // succeded.
        map.on("load", function () {
          map.initialLoaded = true;
        });

        const _this = this;
        map.floorControl = new FloorControl();
        map.floorControl.on("floor-changed", function (args) {
          _this.setOverlayImage(
            args.file
              ? `/* @echo cdn_prefix */maps/overlay/${args.file}`
              : null,
            args.coords
          );
        });
        map.addControl(map.floorControl, "bottom-left");

        _map = map;

        return map;
      },
      // Set the given overlays as available overlay images.
      setFloorOverlays: function (overlays, defaultOverlay) {
        _map.floorControl.updateFloors(overlays, defaultOverlay);
      },
      // Set the currently visible overlay image in the map,
      // or hide it if imgUrl is null.
      setOverlayImage: function (imgUrl, coords) {
        // Even if the map is initialized, it could be that
        // it hasn't loaded yet, so we need to postpone adding
        // the overlay layer.
        // However, the official `loaded()` function is a problem
        // here, because the map is shortly in a "loading" state
        // when source / style is changed, even though the initial
        // loading is complete (and only the initial loading seems
        // to be required to do changes here)
        if (!_map.initialLoaded) {
          const _this = this;
          _map.on("load", function () {
            _this.setOverlayImage(imgUrl, coords);
          });
          return;
        }

        if (imgUrl === null) {
          // Hide overlay
          if (_map.getLayer("overlay-layer"))
            _map.setLayoutProperty("overlay-layer", "visibility", "none");
          if (_map.getLayer("overlay-bg"))
            _map.setLayoutProperty("overlay-bg", "visibility", "none");
        } else {
          const source = _map.getSource("overlay-src");
          if (!source)
            _map.addSource("overlay-src", {
              type: "image",
              url: imgUrl,
              coordinates: coords,
            });
          else
            source.updateImage({
              url: imgUrl,
              coordinates: coords,
            });

          const layer = _map.getLayer("overlay-layer");
          if (!layer) {
            _map.addLayer({
              id: "overlay-bg",
              type: "background",
              paint: {
                "background-color": "#ffffff",
                "background-opacity": 0.6,
              },
            });
            _map.addLayer({
              id: "overlay-layer",
              type: "raster",
              source: "overlay-src",
              paint: {
                "raster-fade-duration": 0,
              },
            });
          } else {
            _map.setLayoutProperty("overlay-layer", "visibility", "visible");
            _map.setLayoutProperty("overlay-bg", "visibility", "visible");
          }
        }
      },
    };
  })()
);
