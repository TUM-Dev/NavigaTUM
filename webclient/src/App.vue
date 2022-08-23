<script lang="ts">
import { RouterView } from "vue-router";
import { setLang, setTheme } from "@/utils/common";
import router from "@/router";
import { onKeyDown, onInput } from "@/modules/autocomplete";

export default {
  data() {
    return {
      search: {
        focused: false,
        keep_focus: false,
        query: "",
        autocomplete: {
          sections: [],
          highlighted: null,
        },
        error: {
          msg: null,
        },
      },
    };
  },
  methods: {
    searchFocus() {
      this.search.focused = true;
      this.search.autocomplete.highlighted = null;
    },
    searchBlur() {
      if (this.search.keep_focus) {
        window.setTimeout(() => {
          // This is relevant if the call is delayed and focused has
          // already been disabled e.g. when clicking on an entry.
          if (this.search.focused) document.getElementById("search")?.focus();
        }, 0);
        this.search.keep_focus = false;
      } else {
        this.search.focused = false;
      }
    },
    searchInput(e) {
      onInput(e.srcElement.value);
    },
    searchKeydown: function (e) {
      onKeyDown(e);
    },
    searchExpand(s) {
      s.expanded = true;
    },
    searchGo(cleanQuery: boolean) {
      if (this.search.query.length === 0) return;

      router.push(`/search?q=${this.search.query}`);
      this.search.focused = false;
      if (cleanQuery) {
        this.search.query = "";
        this.search.autocomplete.sections = [];
      }
      document.getElementById("search")?.blur();
    },
    searchGoTo(id: string, cleanQuery: boolean) {
      // Catch is necessary because vue-router throws an error
      // if navigation is aborted for some reason (e.g. the new
      // url is the same or there is a loop in redirects)
      router.push(`/view/${id}`);
      this.search.focused = false;
      if (cleanQuery) {
        this.search.query = "";
        this.search.autocomplete.sections = [];
      }
      document.getElementById("search")?.blur();
    },
    setLang: setLang,
    setTheme: setTheme,
  },
};
</script>

<template>
  <RouterView />
</template>

<style scoped></style>
