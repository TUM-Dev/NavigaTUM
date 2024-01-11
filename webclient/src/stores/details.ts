import { defineStore } from "pinia";
import type { components } from "@/api_types";
type DetailsResponse = components["schemas"]["DetailsResponse"];
type ImageInfo = components["schemas"]["ImageInfo"];
type RoomfinderMapEntry = components["schemas"]["RoomfinderMapEntry"];
export enum selectedMap {
  interactive,
  roomfinder,
}

export const useDetailsStore = defineStore({
  id: "details",
  state: () => ({
    data: null as DetailsResponse | null,
    image: {
      shown_image: null as ImageInfo | null,
      shown_image_id: null as number | null,
      slideshow_open: false,
    },
    map: {
      // "interactive" is default, because it should show a loading indication.
      selected: selectedMap.interactive as selectedMap,
      roomfinder: {
        selected_id: null as string | null, // Map id
        selected_index: 0 as number, // Index in the 'available' list
      },
    },
  }),
  actions: {
    selectedRoomfinderMap: function (): RoomfinderMapEntry {
      const index = this.map.roomfinder.selected_index;
      return this.data?.maps.roomfinder?.available[index] as RoomfinderMapEntry;
    },
    showImageSlideshow: function (i: number, openSlideshow = true): void {
      if (this.data?.imgs && this.data.imgs[i]) {
        this.image.slideshow_open = openSlideshow;
        this.image.shown_image_id = i;
        this.image.shown_image = this.data.imgs[i];
      } else {
        this.image.slideshow_open = false;
        this.image.shown_image_id = null;
        this.image.shown_image = null;
      }
    },
    loadData: function (d: DetailsResponse): void {
      this.showImageSlideshow(0, false);

      // --- Maps ---
      this.map.selected = d.maps.default === "interactive" ? selectedMap.interactive : selectedMap.roomfinder;
      // Interactive has to be always available, but roomfinder may be unavailable
      if (d.maps.roomfinder !== undefined) {
        // Find default map
        d.maps.roomfinder.available.forEach((availableMap: RoomfinderMapEntry, index: number) => {
          if (availableMap.id === this.data?.maps.roomfinder?.default) {
            this.map.roomfinder.selected_index = index;
            this.map.roomfinder.selected_id = availableMap.id;
          }
        });
      }
      // --- Images ---
      if (d.imgs && d.imgs.length > 0) {
        this.image.shown_image = d.imgs[0];
        this.image.shown_image_id = 0;
      }

      this.data = d;
    },
  },
});
