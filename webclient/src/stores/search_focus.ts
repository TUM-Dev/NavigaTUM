import { defineStore } from "pinia";

export const useSearchBarStore = defineStore({
  id: "search_bar",
  state: () => ({
    focused: false,
  }),
  actions: {
    focus() {
      this.focused = true;
    },
    unfocus() {
      this.focused = false;
    },
  },
});
