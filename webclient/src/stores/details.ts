import { defineStore } from "pinia";
import type { components } from "@/api_types";
type DetailsResponse = components["schemas"]["DetailsResponse"];
type ImageInfo = components["schemas"]["ImageInfo"];
type RoomfinderMapEntry = components["schemas"]["RoomfinderMapEntry"];
export enum selectedMap {
  roomfinder,
  interactive,
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
        x: -1023 - 10, // Outside in top left corner
        y: -1023 - 10,
        modal_open: false,
        modalX: -1023 - 10, // Outside in top left corner
        modalY: -1023 - 10,
        width: 400,
        height: 300,
      },
    },
    coord_picker: {
      // The coordinate picker keeps backups of the subject and body
      // in case someone writes a text and then after that clicks
      // the set coordinate button in the feedback form. If we wouldn't
      // make a backup, this would be lost after clicking confirm there.
      backup_id: null as string | null,
      subject_backup: null as string | null,
      body_backup: null as string | null,
      force_reopen: false,
    },
  }),
  actions: {
    selectedRoomfinderMap: function () {
      const index = this.map.roomfinder.selected_index;
      return this.data?.maps.roomfinder?.available[index];
    },
    showImageSlideshow: function (i: number, openSlideshow = true) {
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
    hideImageSlideshow: function () {
      this.image.slideshow_open = false;
    },
    loadData: function (d: DetailsResponse) {
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
      this.data = d;
    },
  },
});
