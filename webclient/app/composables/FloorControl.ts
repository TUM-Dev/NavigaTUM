import type { IControl, Map as MapLibreMap } from "maplibre-gl";
import { Evented } from "maplibre-gl";
import type { components } from "~/api_types";

type OverlayMapEntryResponse = components["schemas"]["OverlayMapEntryResponse"];
type OverlayMap = {
  readonly default?: number | null;
  readonly available: readonly OverlayMapEntryResponse[];
};

// In reality, this extends maplibregl.Control, but this is apparently not working
export class FloorControl extends Evented implements IControl {
  private readonly container: HTMLDivElement;
  private readonly floor_list: HTMLDivElement;
  private resize_observer: ResizeObserver | undefined;

  constructor() {
    super();

    this.container = document.createElement("div");
    this.container.classList.add("maplibregl-ctrl-group");
    this.container.classList.add("maplibregl-ctrl");
    this.container.classList.add("floor-ctrl");

    // vertical open/collapse button
    const verticalOpenClose = document.createElement("button");
    verticalOpenClose.classList.add("vertical-oc");
    verticalOpenClose.innerHTML = `<span id="vertical-oc-text" /><span class="arrow">▲</span>`;
    verticalOpenClose.addEventListener("click", () => this.container.classList.toggle("closed"));
    // horizontal (primarily on mobile)
    const horizontalOpenClose = document.createElement("button");
    horizontalOpenClose.classList.add("horizontal-oc");
    horizontalOpenClose.innerHTML = `<span id="horizontal-oc-text" /><span class="arrow">❯</span>`;
    horizontalOpenClose.addEventListener("click", () => {
      this.container.classList.toggle("closed");
    });

    this.floor_list = document.createElement("div");
    this.floor_list.id = "floor-list";

    this.container.appendChild(horizontalOpenClose);
    this.container.appendChild(this.floor_list);
    this.container.appendChild(verticalOpenClose);
  }

  onAdd(_map: MapLibreMap): HTMLDivElement {
    // To change on `fullscreen` click on mobile, we need to
    // observe window size changed
    if (ResizeObserver) {
      this.resize_observer = new ResizeObserver(() => {
        this._recalculateLayout(this.floor_list.children.length);
      });
      const interactiveMap = document.getElementById("interactive-map");
      if (interactiveMap) this.resize_observer.observe(interactiveMap);
    }
    return this.container;
  }

  onRemove(): void {
    this.container.remove();
    this.resize_observer?.disconnect();
  }

  public resetFloors(): void {
    this.container.classList.remove("visible");
    this.fire("floor-changed", { file: null, coords: undefined });
  }

  public updateFloors(overlays: OverlayMap): void {
    // `floors` is null or a list of floors with data,
    // `visibleId` is the id of the visible floor.
    this.floor_list.innerHTML = "";

    const clickHandlerBuilder = (
      allFloors: readonly OverlayMapEntryResponse[] | null,
      i: number
    ) => {
      // Because JS
      return () => {
        if (allFloors) {
          // floorlist is reversed, so we need to reverse the index
          const indexInFloorList = allFloors.length - i - 1;
          this._setActiveFloor(indexInFloorList, allFloors[i]?.floor ?? "EG");
          this.fire("floor-changed", {
            file: allFloors[i]?.file,
            coords: allFloors[i]?.coordinates,
          });
        } else {
          this._setActiveFloor(i, "∅");
          this.fire("floor-changed", { file: null, coords: undefined });
        }

        if (!this.container.classList.contains("reduced")) this.container.classList.add("closed");
      };
    };
    let btn: HTMLButtonElement;
    let visibleI = null;
    [...overlays.available]
      .reverse()
      .forEach((floor: OverlayMapEntryResponse, reversed_index: number) => {
        const index = overlays.available.length - reversed_index - 1;
        btn = document.createElement("button");
        btn.innerText = floor.floor;
        btn.addEventListener("click", clickHandlerBuilder(overlays.available, index));
        this.floor_list.appendChild(btn);

        if (floor.id === overlays.default) visibleI = index;
      });

    if (visibleI === null) {
      this._setActiveFloor(this.floor_list.children.length, "∅");
      this.fire("floor-changed", { file: null, coords: undefined });
    } else {
      this._setActiveFloor(visibleI, overlays.available[visibleI]?.floor ?? "EG");
      this.fire("floor-changed", {
        file: overlays.available[visibleI]?.file,
        coords: overlays.available[visibleI]?.coordinates,
      });
    }

    // The last button hides all overlays
    btn = document.createElement("button");
    btn.innerText = "∅";
    btn.addEventListener("click", clickHandlerBuilder(null, this.floor_list.children.length));
    this.floor_list.appendChild(btn);

    this._recalculateLayout(this.floor_list.children.length);

    this.container.classList.add("visible");
  }

  // Recalculate the layout for displaying n floor buttons
  private _recalculateLayout(n: number): void {
    // Calculate required and available size to choose between
    // vertical (default) or horizontal layout
    const mapHeight = document.getElementById("interactive-map")?.clientHeight || 0;
    const topCtrlHeight = document.querySelector(".maplibregl-ctrl-top-left")?.clientHeight || 0;
    const bottomCtrlHeight =
      document.querySelector(".maplibregl-ctrl-bottom-left")?.clientHeight || 0;
    const floorCtrlHeight = document.querySelector(".floor-ctrl")?.clientHeight || 0;

    // The buttons have a height of 29px
    const availableHeight = mapHeight - topCtrlHeight - bottomCtrlHeight + floorCtrlHeight;
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

  private _setActiveFloor(floorListI: number, name: string): void {
    for (let i = 0; i < this.floor_list.children.length; i++) {
      if (i === floorListI) this.floor_list.children[i]?.classList.add("active");
      else this.floor_list.children[i]?.classList.remove("active");
    }
    const vertical = document.getElementById("vertical-oc-text") as HTMLSpanElement;
    vertical.innerText = name;
    const horizontal = document.getElementById("horizontal-oc-text") as HTMLSpanElement;
    horizontal.innerText = name;
  }
}
