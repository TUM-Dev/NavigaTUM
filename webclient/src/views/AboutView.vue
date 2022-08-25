<script setup lang="ts">
import Vue from "vue";
import { h, ref, nextTick } from "vue";
import { useFetch } from "@/utils/fetch";
import { setTitle } from "@/utils/common";

const content = ref(null);
if (!Vue.resolveComponent("md-content")) {
  // c.f. https://stackoverflow.com/questions/47530417/dynamic-router-link
  Vue.defineComponent({
    name: "md-content",
    props: {
      content: {
        type: String,
        required: true,
      },
    },
    render: function () {
      return h(Vue.compile(`<div>${this.content}</div>`));
    },
  });
}
const urlParts = window.location.pathname.split("/");
const lastSegment = urlParts.pop() || urlParts.pop(); // handle trailing slash
const { data, error } = useFetch<string>(`/pages/${lastSegment}.html`, {
  as_text: true,
});
nextTick(() => {
  const e = document.getElementById("view-md");
  if (e === null) {
    console.warn(
      "Failed to update page title. Probably the page is not mounted yet or there was an error."
    );
    return;
  }

  const c = e.firstChild;
  if (c?.firstChild?.nodeName.tagName.toLowerCase() === "h1")
    setTitle(c.firstChild.innerText);
});
</script>

<template>
  <div id="view-md" v-if="content">
    <md-content :content="data"></md-content>

    <!-- This content is here to not purge the spectre css classes -->
    <template v-if="false">
      <pre class="code" data-lang="HTML"><code></code></pre>
    </template>
  </div>
</template>

<style lang="scss" scoped>
@import "../assets/variables";

#view-md {
  padding-top: 15px;

  h1 {
    font-size: 1.8rem;
  }

  h2 {
    font-size: 1.5rem;
  }

  h1,
  h2,
  h3 {
    font-weight: 500;
  }

  code {
    background: $code-bg;
  }

  .code code {
    background: $bg-color;
  }
}
</style>
