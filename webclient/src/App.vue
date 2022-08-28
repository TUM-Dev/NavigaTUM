<script lang="ts">
import AppThemeToggler from "@/components/AppThemeToggler.vue";
import AppLanguageToggler from "@/components/AppLanguageToggler.vue";
import AppSearchBar from "@/components/AppSearchBar.vue";
import { useSearchBarStore } from "@/stores/search_focus";

export default {
  components: { AppSearchBar, AppLanguageToggler, AppThemeToggler },
  data() {
    return {
      search: useSearchBarStore(),
      error: {
        msg: null,
      },
    };
  },
};
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
        <div class="toast toast-error" v-if="error.msg">
          {{ error.msg }}
        </div>
      </div>
    </div>
  </div>

  <!-- Page content container -->
  <div
    id="content"
    class="container grid-lg visible"
    v-bind:class="{ search_focus: search.focused }"
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
                    <a href="https://github.com/TUM-Dev/navigatum">{{
                      $t("footer.sourcecode.text")
                    }}</a>
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
                        ><small>{{ $t("footer.language") }}</small></label
                      >
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

<style scoped></style>
