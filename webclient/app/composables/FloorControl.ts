import type { IControl, Map as MapLibreMap } from "maplibre-gl";
import { Evented } from "maplibre-gl";

interface FloorLevel {
  readonly id: number;
  readonly label: string;
}

// All available floor levels globally
export const FLOOR_LEVELS: readonly FloorLevel[] = [
  { id: 6, label: "6" },
  { id: 5, label: "5" },
  { id: 4, label: "4" },
  { id: 3, label: "3" },
  { id: 2, label: "2" },
  { id: 1, label: "1" },
  { id: 0, label: "EG" },
  { id: -1, label: "-1" },
] as const;

export class FloorControl extends Evented implements IControl {
  private readonly container: HTMLDivElement;
  private readonly floor_list: HTMLDivElement;
  private resize_observer: ResizeObserver | undefined;
  private map: MapLibreMap | undefined;
  /* Optional restriction of available floors */
  private availableFloors: Set<number> = new Set();

  constructor() {
    super();

    this.container = document.createElement("div");
    this.container.classList.add("maplibregl-ctrl-group");
    this.container.classList.add("maplibregl-ctrl");
    this.container.classList.add("floor-ctrl");

    // vertical open/collapse button
    const verticalOpenClose = document.createElement("button");
    verticalOpenClose.classList.add("vertical-oc");
    verticalOpenClose.innerHTML = `<span id="vertical-oc-text">∅</span><span class="arrow">▲</span>`;
    verticalOpenClose.addEventListener("click", () => this.container.classList.toggle("closed"));

    // horizontal (primarily on mobile)
    const horizontalOpenClose = document.createElement("button");
    horizontalOpenClose.classList.add("horizontal-oc");
    horizontalOpenClose.innerHTML = `<span id="horizontal-oc-text">∅</span><span class="arrow">❯</span>`;
    horizontalOpenClose.addEventListener("click", () => {
      this.container.classList.toggle("closed");
    });

    this.floor_list = document.createElement("div");
    this.floor_list.id = "floor-list";

    this.container.appendChild(horizontalOpenClose);
    this.container.appendChild(this.floor_list);
    this.container.appendChild(verticalOpenClose);

    this._renderFloorButtons();
  }

  onAdd(map: MapLibreMap): HTMLDivElement {
    this.map = map;

    // To change on `fullscreen` click on mobile, we need to
    // observe window size changes
    if (ResizeObserver) {
      this.resize_observer = new ResizeObserver(() => {
        this._recalculateLayout(this.floor_list.children.length);
      });
      // Use the map's container element
      const mapContainer = map.getContainer();
      if (mapContainer) this.resize_observer.observe(mapContainer);
    }

    return this.container;
  }

  onRemove(): void {
    this.container.remove();
    this.resize_observer?.disconnect();
    this.map = undefined;
  }

  setAvailableFloors(floorIds: number[]): void {
    this.availableFloors = new Set(floorIds);

    this._renderFloorButtons();
  }

  private _renderFloorButtons(): void {
    this.floor_list.innerHTML = "";

    // Render buttons for each floor level
    FLOOR_LEVELS.forEach((level) => {
      const btn = document.createElement("button");
      btn.innerText = level.label;
      const isAvailable = this.availableFloors.size === 0 || this.availableFloors.has(level.id);

      if (!isAvailable) {
        btn.style.opacity = "0.4";
        btn.style.cursor = "not-allowed";
        btn.style.color = "#999";
      }

      btn.addEventListener("click", () => {
        if (!isAvailable) return;
        this.setLevel(level.id);
        if (!this.container.classList.contains("reduced")) {
          this.container.classList.add("closed");
        }
      });
      this.floor_list.appendChild(btn);
    });

    // Add "hide all" button
    const hideBtn = document.createElement("button");
    hideBtn.innerText = "∅";
    hideBtn.classList.add("active"); // Start with all floors hidden
    hideBtn.addEventListener("click", () => {
      this.setLevel(null);
      if (!this.container.classList.contains("reduced")) {
        this.container.classList.add("closed");
      }
    });
    this.floor_list.appendChild(hideBtn);

    this._recalculateLayout(this.floor_list.children.length);
  }

  setLevel(level: number | null): void {
    // Update button states
    const buttons = this.floor_list.children;
    for (let i = 0; i < buttons.length; i++) {
      const button = buttons[i];
      if (button) {
        if (i === buttons.length - 1 && level === null) {
          // Last button (hide all) is active
          button.classList.add("active");
        } else if (i < FLOOR_LEVELS.length && FLOOR_LEVELS[i]?.id === level) {
          // This level button is active
          button.classList.add("active");
        } else {
          button.classList.remove("active");
        }
      }
    }

    // Update open/close button text
    const displayText =
      level === null ? "∅" : (FLOOR_LEVELS.find((l) => l.id === level)?.label ?? "∅");
    const vertical = document.getElementById("vertical-oc-text");
    if (vertical) vertical.innerText = displayText;
    const horizontal = document.getElementById("horizontal-oc-text");
    if (horizontal) horizontal.innerText = displayText;

    // Update layer visibility
    this._updateLayerVisibility(level);

    // Fire event
    this.fire("level-changed", { level });
  }

  private _updateLayerVisibility(level: number | null): void {
    if (!this.map) return;

    // Hide all floor layers
    for (const floorLevel of FLOOR_LEVELS) {
      const layerId = `floor-level-${floorLevel.id}`;
      if (this.map.getLayer(layerId)) {
        this.map.setLayoutProperty(layerId, "visibility", "none");
      }
    }

    // Show the selected level
    if (level !== null) {
      const layerId = `floor-level-${level}`;
      if (this.map.getLayer(layerId)) {
        this.map.setLayoutProperty(layerId, "visibility", "visible");
      }
    }
  }

  private _recalculateLayout(n: number): void {
    // Calculate required and available size to choose between
    // vertical (default) or horizontal layout
    const mapHeight = this.map?.getContainer()?.clientHeight || 0;
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
      this.container.classList.remove("closed");
      this.container.classList.remove("horizontal");
      this.container.classList.add("reduced");
    } else {
      this.container.classList.remove("reduced");
      this.container.classList.add("closed");

      // 25px = 10px reserved for top/bottom margin + 5px between control groups
      // 29px = additional height from the open/collapse button
      if (availableHeight - (requiredHeight + 29) > 25) {
        this.container.classList.remove("horizontal");
      } else {
        this.container.classList.add("horizontal");
      }
    }
  }
}
