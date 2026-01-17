import type { IControl, Map as MapLibreMap } from "maplibre-gl";
import { Evented } from "maplibre-gl";

export class CombinedControlGroup extends Evented implements IControl {
  private readonly _container: HTMLDivElement = document.createElement("div");
  private controls: IControl[] = [];
  // DOM containers returned by onAdd()
  private containers: HTMLElement[] = [];
  private map: undefined | MapLibreMap = undefined;

  constructor(controls: IControl[]) {
    super();
    this.controls = controls;
  }

  onAdd(map: MapLibreMap): HTMLDivElement {
    this.map = map;
    this._container.className = "maplibregl-ctrl maplibregl-ctrl-group";

    for (const ctrl of this.controls) {
      const ctrlContainer = ctrl.onAdd(map);
      this.containers.push(ctrlContainer);

      // Extract buttons from the control's container and add to our wrapper
      const buttons = ctrlContainer.querySelectorAll("button");
      buttons.forEach((button) => {
        this._container.appendChild(button);
      });
    }

    return this._container;
  }

  onRemove(): void {
    if (this.map) {
      for (const ctrl of this.controls) ctrl.onRemove?.(this.map);
    }
    if (this._container) this._container.remove();
  }
}
