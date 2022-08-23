<script lang="ts">
function mdNavigateTo(to, from, next, component) {
  navigatum.beforeNavigate(to, from);

  // Component is not registered on pageload because Vue might not be availabe then
  if (!Vue.component("md-content")) {
    // c.f. https://stackoverflow.com/questions/47530417/dynamic-router-link
    Vue.component("md-content", {
      props: {
        content: {
          type: String,
          required: true,
        },
      },
      render: function (h) {
        return h(Vue.compile(`<div>${this.content}</div>`));
      },
    });
  }

  /* global cachedFetch */
  cachedFetch
    .fetch(`/* @echo app_prefix */pages/${to.params.name}.html`, {
      as_text: true,
    })
    .then((resp) => {
      if (component) {
        next();
        navigatum.afterNavigate(to, from);
        component.loadPage(resp);
      } else {
        next((vm) => {
          navigatum.afterNavigate(to, from);
          vm.loadPage(resp);
        });
      }
    });
}

navigatum.registerView("md", {
  name: "view-md",
  template: { gulp_inject: "view-md.inc" },
  data: function () {
    return {
      content: null,
    };
  },
  beforeRouteEnter: function (to, from, next) {
    mdNavigateTo(to, from, next, null);
  },
  beforeRouteUpdate: function (to, from, next) {
    mdNavigateTo(to, from, next, this);
  },
  methods: {
    loadPage: function (content) {
      this.content = content;

      this.$nextTick(() => {
        const e = document.getElementById("view-md");
        if (e === null) {
          console.warn(
            "Failed to update page title. Probably the page is not mounted yet or there was an error."
          );
          return;
        }

        const c = e.firstChild;
        if (c && c.firstChild.tagName.toLowerCase() === "h1")
          navigatum.setTitle(c.firstChild.innerText);
      });
    },
  },
});
</script>


<style lang="scss">
@import "@assets/variables";

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


<template>
  <div id="view-md" v-if="content">
  <md-content :content="content"></md-content>

  <!-- This content is here to not purge the spectre css classes -->
  <template v-if="false">
      <pre class="code" data-lang="HTML"><code></code></pre>
    </template>
  </div>
</template>
