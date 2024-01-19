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
    showImageSlideshow: function (openSlideshow: boolean): void {
      this.image.slideshow_open = this.data?.imgs ? openSlideshow : false;
    },
    loadData: function (d: DetailsResponse): void {
      this.showImageSlideshow(false);

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
      } else {
        this.image.shown_image = null;
      }

      this.data = d;
    },
  },
});
