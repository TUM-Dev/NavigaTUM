<script lang="ts">
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
      },
      error: {
        msg: null,
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
  <header class="navbar" id="navbar">
        <div class="container grid-lg">
          <div class="columns">
            <div class="column hide-lg">
              <RouterLink to="/">
              <img v-bind:alt="$t('meta.logo_alt')"
                   src="/logo.svg"
                   id="logo"/>  <!-- 7px for logo1 -->
              </RouterLink>
            </div>
            <div class="column col-8 col-lg-11 col-mx-auto">
              <div class="form-autocomplete">
                <div class="input-group has-icon-left">
                  <input id="search"
                         type="text"
                         class="form-input input-lg"
                         v-bind:placeholder="$t('search.placeholder')"
                         v-model="search.query"
                         @input="searchInput"
                         @focus="searchFocus"
                         @blur="searchBlur"
                         @keydown="searchKeydown"
                         autocomplete="off"
                         v-bind:aria-label="$t('search.aria-searchlabel')">
                  <i class="form-icon icon icon-search"></i>
                  <button class="btn btn-primary input-group-btn btn-lg"
                          @click="searchGo(false)"
                          v-bind:aria-label="$t('search.aria-actionlabel')">
                    {{ $t("search.action") }}
                  </button>
                </div>
                <!-- Autocomplete -->
                <ul class="menu"
                    v-bind:class="{'d-none': !search.focused || (search.autocomplete.sections.length == 0)}"
                    v-cloak>

                  <!--<li class="search-comment filter">
                    Suche einschränken auf:
                    <a class="bt btn-link btn-sm">Räume</a>
                  </li>-->

                  <template v-for="s in search.autocomplete.sections">
                    <li class="divider"
                        DISABLED-v-if="search.autocomplete.sections.length > 1"
                        v-bind:data-content="s.name"></li>
                    <li v-for="(e, i) in s.entries"
                        v-if="s.n_visible === undefined || (s.n_visible !== undefined && i < s.n_visible) || s.expanded"
                        class="menu-item">
                      <a v-bind:class="{active: e.id == search.autocomplete.highlighted}"
                         @click="searchGoTo(e.id, true)"
                         @mousedown="search.keep_focus = true"
                         @mouseover="search.autocomplete.highlighted = null">
                        <div class="tile">
                          <div class="tile-icon">
                            <template v-if="e.type == 'room' || e.type == 'virtual_room'">
                               <i v-if="e.parsed_id" class="icon icon-search"></i>
                               <i v-else class="icon icon-location"></i>
                            </template>
                            <img v-else
                                 src="@/assets/thumb-building.webp"
                                 class="avatar avatar-sm">
                          </div>
                          <div class="tile-content">
                            <span class="tile-title">
                              <span v-if="e.parsed_id" v-html="e.parsed_id"></span>
                              <i v-if="e.parsed_id" class="icon icon-caret"></i>
                              <span v-html="e.name" v-bind:style="{'opacity': e.parsed_id ? 0.5 : 1}"></span>
                            </span>
                            <small class="tile-subtitle text-gray">
                              {{ e.subtext }}<template v-if="e.subtext_bold">, <b v-html="e.subtext_bold"></b></template>
                            </small>
                          </div>
                        </div>
                      </a>
                      <!--<div class="menu-badge">
                        <label class="label label-primary">2</label>
                      </div>-->
                    </li>

                    <li class="search-comment nb_results">
                      <a v-if="!s.expanded && s.n_visible < s.entries.length"
                         @mousedown="search.keep_focus = true"
                         @click="searchExpand(s)">
                        +{{ s.entries.length - s.n_visible }} {{ $t("search.hidden") }},
                      </a>
                      <template>
                        {{ s.estimatedTotalHits > 20 ? $t('search.approx') : "" }}{{ s.estimatedTotalHits }}
                        {{ s.estimatedTotalHits === 1 ? $t('search.result') : $t('search.results') }}
                      </template>
                    </li>
                  </template>

                  <!--<li class="search-comment actions">
                    <div>
                      <button class="btn btn-sm">
                        <i class="icon icon-arrow-right"></i> in Gebäude Suchen
                      </button>
                    </div>
                    <div>
                      <button class="btn btn-sm">
                        <i class="icon icon-location"></i> Hörsäle
                      </button>
                    </div>
                    <div>
                      <button class="btn btn-sm">
                        <i class="icon icon-location"></i> Seminarräume
                      </button>
                    </div>
                  </li>-->

                  <!--<li class="divider" data-content="Veranstaltungen"></li>
                  <li class="menu-item">
                    <a href="#">
                      <div class="tile">
                        <div class="tile-icon">
                          <i class="icon icon-time"></i>
                        </div>
                        <div class="tile-content">
                          <span class="tile-title">
                            Advanced Practical Course Games Engineering: Building Information Modeling (IN7106)
                          </span>
                          <small class="tile-subtitle text-gray">
                            Übung mit 4 Gruppen
                          </small>
                        </div>
                      </div>
                    </a>
                    <div class="menu-badge" style="display: none;">
                      <label class="label label-primary">frei</label>
                    </div>
                  </li>-->
                </ul>
              </div>
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
      <div id="content"
           class="container grid-lg visible"
           v-bind:class="{ search_focus: search.focused }">
        <div class="columns">
          <div class="column col-lg-11 col-mx-auto">
            <RouterView/>
          </div>
        </div>
      </div>

        <noscript>
          <div id="content-header" class="container grid-lg">
            <div class="columns">
              <div class="column col-lg-11 col-mx-auto">
                <div class="toast toast-error">
                  {{ $t("core_js.error.noscript.js_required") }}<br>
                  {{ $t("core_js.error.noscript.please_enable_js") }}<br>
                  <br>
                  {{ $t("core_js.error.noscript.continue_with_different_useragent") }}
                </div>
              </div>
            </div>
          </div>
        </noscript>
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
                        <a href="https://github.com/TUM-Dev/navigatum">{{ $t("footer.sourcecode.text") }}</a>
                      </li>
                      <li>
                        <RouterLink to="api">{{ $t("footer.api.text") }}</RouterLink>
                      </li>
                      <li>
                        <RouterLink v-bind:to="'/about/'+ $t('footer.about.link')">{{ $t("footer.about.text") }}</RouterLink>
                      </li>
                    </ul>
                    <ul class="column">
                      <li>
                        <button onclick="openFeedback()" class="btn btn-link" aria-label="Open the feedback-form">{{ $t("footer.feedback.text") }}</button>
                      </li>
                      <li>
                        <RouterLink v-bind:to="'/about/'+$t('footer.privacy.link')">{{ $t("footer.privacy.text") }}</RouterLink>
                      </li>
                      <li>
                        <RouterLink v-bind:to="'/about/'+ $t('footer.imprint.link')">{{ $t("footer.imprint.text") }}</RouterLink>
                      </li>
                    </ul>
                  </div>
                </div>
                <div class="column col-auto col-ml-auto col-xs-12 settings">
                  <div class="show-xs divider" style="height: 20px;"></div>
                  <div class="columns">
                    <div class="column col-12 col-xs-8 col-mx-auto">
                      <div class="columns setting-group">
                        <div class="column col">
                          <label for="setting-lang"><small>{{ $t("footer.language") }}</small></label>
                        </div>
                        <div class="column col-auto">
                          <div class="btn-group btn-group-block" id="setting-lang">
                            <!-- @if "${{_lang_}}$"="de" -->
                            <button class="btn btn-sm active" disabled>DE</button>
                            <button class="btn btn-sm" @click="setLang('en')">EN</button>
                            <!-- @endif -->
                            <!-- @if "${{_lang_}}$"="en" -->
                            <button class="btn btn-sm" @click="setLang('de')">DE</button>
                            <button class="btn btn-sm active" disabled>EN</button>
                            <!-- @endif -->
                          </div>
                        </div>
                      </div>
                      <div class="columns setting-group">
                        <div class="column col">
                          <label for="setting-theme"><small>{{ $t("footer.theme") }}</small></label>
                        </div>
                        <div class="column col-auto">
                          <div class="btn-group btn-group-block" id="setting-theme">
                            <!-- @if theme="light" -->
                            <button class="btn btn-sm active" disabled>{{ $t("footer.theme_light") }}</button>
                            <button class="btn btn-sm" @click="setTheme('dark')">{{ $t("footer.theme_dark") }}</button>
                            <!-- @endif -->
                            <!-- @if theme="dark" -->
                            <button class="btn btn-sm" @click="setTheme('light')">{{ $t("footer.theme_light") }}</button>
                            <button class="btn btn-sm active" disabled>{{ $t("footer.theme_dark") }}</button>
                            <!-- @endif -->
                          </div>
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
