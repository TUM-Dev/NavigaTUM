<script setup lang="ts">
import AppSearchBar from "@/components/AppSearchBar.vue";
import AppLanguageToggler from "@/components/AppLanguageToggler.vue";
import AppThemeToggler from "@/components/AppThemeToggler.vue";
import { useGlobalStore } from "@/stores/global";

const global = useGlobalStore();
</script>

<template>
  <header class="navbar" id="navbar">
    <div class="container grid-lg">
      <div class="columns">
        <div class="column hide-lg">
          <RouterLink to="/">
            <img
              v-bind:alt="$t('meta.logo_alt')"
              src="./assets/logo.svg"
              id="logo"
            />
            <!-- 7px for logo1 -->
          </RouterLink>
        </div>
        <div class="column col-8 col-lg-11 col-mx-auto">
          <AppSearchBar></AppSearchBar>
        </div>
      </div>
    </div>
  </header>

  <!-- General error message toast -->
  <div id="content-header" class="container grid-lg" v-cloak>
    <div class="columns">
      <div class="column col-lg-11 col-mx-auto">
        <div class="toast toast-error" v-if="global.error_message">
          {{ global.error_message }}
        </div>
      </div>
    </div>
  </div>

  <!-- Page content container -->
  <div
    id="content"
    class="container grid-lg visible"
    v-bind:class="{ search_focus: global.search_focused }"
  >
    <div class="columns">
      <div class="column col-lg-11 col-mx-auto">
        <RouterView />
      </div>
    </div>
  </div>
  <!-- Loading indicator -->
  <div id="loading-page" v-cloak>
    <div class="loading loading-lg"></div>
  </div>

  <div style="margin-bottom: 30px"></div>
  <div style="padding-bottom: 70px" class="show-xs"></div>

  <!-- Footer -->
  <footer>
    <div class="container grid-lg">
      <div class="columns">
        <div class="column col-lg-11 col-mx-auto">
          <div class="columns">
            <div class="column col-auto col-xs-12 links">
              <div class="columns">
                <ul class="column col-auto">
                  <li>
                    <a href="https://github.com/TUM-Dev/navigatum">
                      {{ $t("footer.sourcecode.text") }}
                    </a>
                  </li>
                  <li>
                    <RouterLink to="api">
                      {{ $t("footer.api.text") }}
                    </RouterLink>
                  </li>
                  <li>
                    <RouterLink v-bind:to="'/about/' + $t('footer.about.link')">
                      {{ $t("footer.about.text") }}
                    </RouterLink>
                  </li>
                </ul>
                <ul class="column">
                  <li>
                    <button
                      onclick="openFeedback()"
                      class="btn btn-link"
                      aria-label="Open the feedback-form"
                    >
                      {{ $t("footer.feedback.text") }}
                    </button>
                  </li>
                  <li>
                    <RouterLink
                      v-bind:to="'/about/' + $t('footer.privacy.link')"
                    >
                      {{ $t("footer.privacy.text") }}
                    </RouterLink>
                  </li>
                  <li>
                    <RouterLink
                      v-bind:to="'/about/' + $t('footer.imprint.link')"
                    >
                      {{ $t("footer.imprint.text") }}
                    </RouterLink>
                  </li>
                </ul>
              </div>
            </div>
            <div class="column col-auto col-ml-auto col-xs-12 settings">
              <div class="show-xs divider" style="height: 20px"></div>
              <div class="columns">
                <div class="column col-12 col-xs-8 col-mx-auto">
                  <div class="columns setting-group">
                    <div class="column col">
                      <label for="setting-lang"
                        ><small>{{ $t("footer.language") }}</small>
                      </label>
                    </div>
                    <div class="column col-auto">
                      <AppLanguageToggler />
                    </div>
                  </div>
                  <div class="columns setting-group">
                    <div class="column col">
                      <label for="setting-theme">
                        <small>{{ $t("footer.theme") }}</small>
                      </label>
                    </div>
                    <div class="column col-auto">
                      <AppThemeToggler />
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </footer>
</template>

<style lang="scss">
@import "./assets/variables";

/* === Navbar === */
#navbar {
  padding: 10px 0;
  box-shadow: 0 2px 3px $header-shadow-color;
  width: 100%;
  position: fixed;
  background: $header-color;
  top: 0;
  z-index: 2000;

  #logo {
    height: 24px;
    margin-top: 9px;
  }
  .input-group button {
    border: 0;
  }
}

/* === Content === */
// 10px + 60px for header
#content-header {
  margin-top: 70px;
}

#content.visible {
  /* For some reason (I assume because the 'visible' class is not set when vue loads),
     * this class gets removed if vue adds/removes the 'search_focus' class. For this reason
     * opacity on page navigation is set as style property in JS. It is only guaranteed that
     * this class is there on page-load. */
  transition: opacity 0.07s;
}

#content.search_focus {
  opacity: 0.7;
}

/* === Footer === */
footer {
  padding: 8px 0 16px;
  background: $footer-color;
  bottom: -130px;
  position: absolute;
  left: 0;
  right: 0;
  text-align: center;

  .links {
    text-align: left;

    ul {
      margin: 0;

      li {
        list-style: none;
        margin-top: 0;
      }
    }

    a,
    RouterLink,
    button {
      font-size: 0.6rem;
    }

    button {
      height: auto;
      padding: 0;
    }

    button:hover {
      text-decoration: underline;
    }
  }

  .settings {
    .setting-group {
      margin-top: calc(0.4rem - 1px);
    }

    .btn-group {
      min-width: 110px;

      .btn {
        border-color: transparent;

        &:disabled {
          background-color: $footer-setting-bg-disabled;
          color: $footer-color;
        }
      }
    }
  }
}

// 'xs' (mobile)
@media (max-width: 480px) {
  footer {
    bottom: -200px;

    .links {
      ul {
        margin: 0.8rem;

        li {
          margin-top: 0.4rem;
        }
      }

      a,
      RouterLink,
      button {
        font-size: 0.7rem;
      }
    }
  }
}
</style>
