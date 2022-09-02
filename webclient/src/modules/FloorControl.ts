import { Evented } from "mapbox-gl";
import type { Map, IControl } from "mapbox-gl";
import type { OverlayMap, OverlayMapEntry } from "@/codegen";

// In reality, this extends mapboxgl.Control, but this is apparently not working
export class FloorControl extends Evented implements IControl {
  private readonly container: HTMLDivElement;
  private readonly floor_list: HTMLDivElement;
  private resize_observer: ResizeObserver | undefined;
  private map: Map | undefined;

  constructor() {
    super();

    this.container = document.createElement("div");
    this.container.classList.add("mapboxgl-ctrl-group");
    this.container.classList.add("mapboxgl-ctrl");
    this.container.classList.add("floor-ctrl");

    // vertical open/collapse button
    const verticalOpenClose = document.createElement("button");
    verticalOpenClose.classList.add("vertical-oc");
    verticalOpenClose.innerHTML =
      "<span id='vertical-oc-text'></span><span class='arrow'>▲</span>";
    verticalOpenClose.addEventListener("click", () =>
      this.container.classList.toggle("closed")
    );
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
  }

  onAdd(map: Map) {
    this.map = map;

    // To change on `fullscreen` click on mobile, we need to
    // observe window size changed
    if (ResizeObserver) {
      this.resize_observer = new ResizeObserver(() => {
        this._recalculateLayout(this.floor_list.children.length);
      });
      this.resize_observer.observe(document.getElementById("interactive-map")!);
    }
    return this.container;
  }

  onRemove() {
    this.container.remove();
    this.map = undefined;
  }

  resetFloors() {
    this.container.classList.remove("visible");
    this.fire("floor-changed", { file: null, coords: undefined });
  }
  updateFloors(overlays: OverlayMap) {
    // `floors` is null or a list of floors with data,
    // `visibleId` is the id of the visible floor.
    this.floor_list.innerHTML = "";

    const _this = this;
    const clickHandlerBuilder = function (
      allFloors: Array<OverlayMapEntry> | null,
      i: number
    ) {
      // Because JS
      return () => {
        if (allFloors) {
          _this._setActiveFloor(i, allFloors[i].floor);
          _this.fire("floor-changed", {
            file: allFloors[i].file,
            coords: allFloors[i].coordinates,
          });
        } else {
          _this._setActiveFloor(i, "∅");
          _this.fire("floor-changed", { file: null, coords: undefined });
        }

        if (!_this.container.classList.contains("reduced"))
          _this.container.classList.add("closed");
      };
    };
    let btn;
    let visibleI = null;
    overlays.available.reverse().forEach((floor, index: number) => {
      btn = document.createElement("button");
      btn.innerText = floor.floor;
      btn.addEventListener(
        "click",
        clickHandlerBuilder(overlays.available, index)
      );
      this.floor_list.appendChild(btn);

      if (floor.id === overlays.default) visibleI = index;
    });

    if (visibleI === null) {
      this._setActiveFloor(this.floor_list.children.length, "∅");
      this.fire("floor-changed", { file: null, coords: undefined });
    } else {
      this._setActiveFloor(visibleI, overlays.available[visibleI].floor);
      this.fire("floor-changed", {
        file: overlays.available[visibleI].file,
        coords: overlays.available[visibleI].coordinates,
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

  // Recalculate the layout for displaying n floor buttons
  private _recalculateLayout(n: number) {
    // Calculate required and available size to choose between
    // vertical (default) or horizontal layout
    const mapHeight = document.getElementById("interactive-map")!.clientHeight;
    const topCtrlHeight = document.querySelector(
      ".mapboxgl-ctrl-top-left"
    )!.clientHeight;
    const bottomCtrlHeight = document.querySelector(
      ".mapboxgl-ctrl-bottom-left"
    )!.clientHeight;
    const floorCtrlHeight = document.querySelector(".floor-ctrl")!.clientHeight;

    // The buttons have a height of 29px
    const availableHeight =
      mapHeight - topCtrlHeight - bottomCtrlHeight + floorCtrlHeight;
    const requiredHeight = 29 * n;

    // 3 or fewer buttons can always be displayed in reduced layout.
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
  }

  private _setActiveFloor(floorListI: number, name: string) {
    for (let i = 0; i < this.floor_list.children.length; i++) {
      if (i === floorListI) this.floor_list.children[i].classList.add("active");
      else this.floor_list.children[i].classList.remove("active");
    }
    document.getElementById("vertical-oc-text")!.innerText = name;
    document.getElementById("horizontal-oc-text")!.innerText = name;
  }
}
