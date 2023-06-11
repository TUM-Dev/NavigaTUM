<script setup lang="ts">
import AppLanguageToggler from "@/components/AppLanguageToggler.vue";
import AppThemeToggler from "@/components/AppThemeToggler.vue";
import { useGlobalStore } from "@/stores/global";
const global = useGlobalStore();
</script>
<template>
  <footer data-cy="main-footer">
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
                    <RouterLink to="/api">
                      {{ $t("footer.api.text") }}
                    </RouterLink>
                  </li>
                  <li>
                    <RouterLink :to="'/about/' + $t('footer.about.link')">
                      {{ $t("footer.about.text") }}
                    </RouterLink>
                  </li>
                </ul>
                <ul class="column">
                  <li>
                    <button
                      data-cy="open-feedback-footer"
                      @click="global.openFeedback()"
                      class="btn btn-link"
                      :aria-label="$t('footer.feedback.open')"
                    >
                      {{ $t("footer.feedback.text") }}
                    </button>
                  </li>
                  <li>
                    <RouterLink :to="'/about/' + $t('footer.privacy.link')">
                      {{ $t("footer.privacy.text") }}
                    </RouterLink>
                  </li>
                  <li>
                    <RouterLink :to="'/about/' + $t('footer.imprint.link')">
                      {{ $t("footer.imprint.text") }}
                    </RouterLink>
                  </li>
                </ul>
              </div>
            </div>
            <div class="column col-auto col-ml-auto col-xs-12 settings">
              <div class="show-xs divider" style="height: 20px" />
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
