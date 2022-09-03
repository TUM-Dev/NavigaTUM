import { defineStore } from "pinia";
import type { DetailsResponse } from "@/codegen";
export enum selectedMap {
  roomfinder,
  interactive,
}

export const useDetailsStore = defineStore({
  id: "details",
  state: () => ({
    data: null as DetailsResponse | null,
    map: {
      // "interactive" is default, because it should show a loading indication.
      selected: selectedMap.interactive,
      roomfinder: {
        selected_id: null, // Map id
        selected_index: null, // Index in the 'available' list
        x: -1023 - 10, // Outside in top left corner
        y: -1023 - 10,
        width: 400,
        height: 300,
      },
    },
  }),
});
