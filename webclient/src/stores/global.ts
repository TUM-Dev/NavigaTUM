import { defineStore } from "pinia";

export const useGlobalStore = defineStore({
  id: "global",
  state: () => ({
    search_focused: false,
    error_message: null,
  }),
  actions: {
    focus_search() {
      this.search_focused = true;
    },
    unfocus_search() {
      this.search_focused = false;
    },
  },
});
