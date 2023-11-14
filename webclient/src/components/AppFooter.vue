<script setup lang="ts">
import AppLanguageToggler from "@/components/AppLanguageToggler.vue";
import AppThemeToggler from "@/components/AppThemeToggler.vue";
import { useGlobalStore } from "@/stores/global";
import { useI18n } from "vue-i18n";
const global = useGlobalStore();
const theme = (localStorage.getItem("theme") || "light") as "light" | "dark";
const lang = (localStorage.getItem("lang") || "de") as "de" | "en";
// If we do not include the image here like this, vite/rollup is unable to load it
const brandLogo = new URL(`/src/assets/logos/tum_${theme}_${lang}.svg`, import.meta.url);
const { t } = useI18n({ useScope: "local" });
</script>
<template>
  <footer data-cy="main-footer">
    <div class="container grid-lg">
      <div class="columns">
        <div class="column col-lg-11 col-mx-auto">
          <div class="columns">
            <div class="column col-auto col-xs-12 links">
              <div class="columns">
                <ul class="column">
                  <li>
                    <a href="https://github.com/TUM-Dev/navigatum">
                      {{ t("sourcecode.text") }}
                    </a>
                  </li>
                  <li>
                    <RouterLink to="/api">
                      {{ t("api.text") }}
                    </RouterLink>
                  </li>
                  <li>
                    <RouterLink :to="'/about/' + t('about.link')">
                      {{ t("about.text") }}
                    </RouterLink>
                  </li>
                </ul>
                <ul class="column">
                  <li>
                    <button
                      data-cy="open-feedback-footer"
                      @click="global.openFeedback()"
                      class="btn btn-link"
                      :aria-label="t('feedback.open')"
                    >
                      {{ t("feedback.text") }}
                    </button>
                  </li>
                  <li>
                    <RouterLink :to="'/about/' + t('privacy.link')">
                      {{ t("privacy.text") }}
                    </RouterLink>
                  </li>
                  <li>
                    <RouterLink :to="'/about/' + t('imprint.link')">
                      {{ t("imprint.text") }}
                    </RouterLink>
                  </li>
                </ul>
              </div>
            </div>
            <div class="column hide-sm official_roomfinder">
              {{ t("official_roomfinder") }}<br />
              <a href="https://tum.de" target="_blank">
                <img :alt="t('tum_logo_alt')" :src="brandLogo.href" width="200" class="mx-auto" />
              </a>
            </div>
            <div class="column col-auto col-ml-auto col-xs-12 settings">
              <div class="columns">
                <div class="column col-12 col-xs-8 col-mx-auto">
                  <div class="columns setting-group">
                    <div class="column col">
                      <label for="setting-lang"
                        ><small>{{ t("language") }}</small>
                      </label>
                    </div>
                    <div class="column col-auto">
                      <AppLanguageToggler />
                    </div>
                  </div>
                  <div class="columns setting-group">
                    <div class="column col">
                      <label for="setting-theme">
                        <small>{{ t("theme") }}</small>
                      </label>
                    </div>
                    <div class="column col-auto">
                      <AppThemeToggler />
                    </div>
                  </div>
                </div>
              </div>
            </div>
            <div class="column col-12 show-sm official_roomfinder mt-3">
              {{ t("official_roomfinder") }}<br />
              <a href="https://tum.de" target="_blank">
                <img :alt="t('tum_logo_alt')" :src="brandLogo.href" width="200" class="mx-auto" />
              </a>
            </div>
          </div>
        </div>
      </div>
    </div>
  </footer>
</template>

<style lang="scss">
@import "@/assets/variables";

footer {
  margin-top: 30px;
  padding: 8px 0 16px;
  background: $footer-color;
  position: relative;
  left: 0;
  right: 0;
  top: 0;
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
  .official_roomfinder {
    font-size: 0.6rem;
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
          color: $footer-setting-color-disabled;
        }
      }
    }
  }
}

// 'xs' (mobile)
@media (max-width: 480px) {
  footer {
    margin-top: 50px;
    bottom: -200px;

    .links,
    .settings,
    .official_roomfinder {
      margin-top: 0.8rem;
      margin-bottom: 0.8rem;
    }

    .links {
      ul {
        margin-left: 0.8rem;
        margin-right: 0.8rem;
        li {
          margin-top: 0.4rem;
          text-align: center !important;
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

<i18n lang="yaml">
de:
  about:
    link: ueber-uns
    text: Über uns
  api:
    link: api
    text: API
  feedback:
    open: Feedback Form öffnen
    text: Feedback senden
  imprint:
    link: impressum
    text: Impressum
  language: Sprache
  official_roomfinder: Offizieller Roomfinder
  privacy:
    link: datenschutz
    text: Datenschutz
  sourcecode:
    text: Source Code
  theme: Theme
  tum_logo_alt: The Logo of the Technical University Munich
en:
  about:
    link: about-us
    text: About us
  api:
    link: api
    text: API
  feedback:
    open: Open the feedback-form
    text: Feedback
  imprint:
    link: impressum
    text: Imprint
  language: Language
  official_roomfinder: Official roomfinder
  privacy:
    link: privacy
    text: Privacy
  sourcecode:
    text: Source Code
  theme: Theme
  tum_logo_alt: Das Logo der Technischen Universität München
</i18n>
