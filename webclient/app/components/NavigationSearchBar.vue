<script setup lang="ts">
import { mdiCrosshairsGps } from "@mdi/js";
import { useGeolocation } from "@vueuse/core";
import { useRouteQuery } from "@vueuse/router";
import type { operations } from "~/api_types";

type SearchResponse = operations["search_handler"]["responses"][200]["content"]["application/json"];

const props = defineProps<{
  queryId: string;
}>();
const { t, locale } = useI18n({ useScope: "local" });
const route = useRoute();
const router = useRouter();
const currently_actively_picking = ref(false);
const { coords, locatedAt, error: geoError, resume, pause } = useGeolocation();
const isGettingLocation = ref(false);

const isGeolocationSupported = computed(() => {
  return process.client && typeof navigator !== "undefined" && "geolocation" in navigator;
});

const query = useRouteQuery<string>(`q_${props.queryId}`, "", {
  mode: "replace",
  route,
  router,
});
const selected = useRouteQuery<string>(props.queryId, "", {
  mode: "replace",
  route,
  router,
});
const highlighted = ref<number>(0);
const sites_buildings_expanded = ref<boolean>(false);

const visibleElements = computed<string[]>(() => {
  if (!data.value) return [];

  const visible: string[] = [];
  for (const section of data.value.sections) {
    if (section.facet === "sites_buildings") {
      const max_sites_buildings = sites_buildings_expanded.value ? Number.POSITIVE_INFINITY : section.n_visible;
      visible.push(...section.entries.slice(0, max_sites_buildings).map((e) => e.id));
    } else visible.push(...section.entries.map((e) => e.id));
  }
  return visible;
});

function select(id: string) {
  currently_actively_picking.value = false;
  selected.value = id;
  for (const section of data.value?.sections ?? []) {
    for (const entry of section.entries) {
      if (entry.id === id) {
        query.value = entry.name.replaceAll("<b class='text-blue'>", "").replaceAll("</b>", "").trim();
      }
    }
  }
}

function useCurrentLocation() {
  if (!import.meta.client || typeof navigator === "undefined" || !("geolocation" in navigator)) {
    alert(t("gps.error.not_supported"));
    return;
  }

  // Check if running on HTTPS or localhost
  const isSecureContext = window.location.protocol === "https:" || window.location.hostname === "localhost";
  if (!isSecureContext) {
    alert(t("gps.error.https_required"));
    return;
  }

  isGettingLocation.value = true;

  // Use native geolocation API directly to maintain user gesture connection
  navigator.geolocation.getCurrentPosition(
    (position) => {
      // Success callback
      query.value = t("gps.my_location");
      selected.value = `${position.coords.latitude},${position.coords.longitude}`;
      currently_actively_picking.value = false;
      isGettingLocation.value = false;
    },
    (error) => {
      // Error callback
      isGettingLocation.value = false;

      switch (error.code) {
        case error.PERMISSION_DENIED:
          alert(t("gps.error.permission_denied"));
          break;
        case error.POSITION_UNAVAILABLE:
          alert(t("gps.error.position_unavailable"));
          break;
        case error.TIMEOUT:
          alert(t("gps.error.timeout"));
          break;
        default:
          alert(t("gps.error.general"));
      }
    },
    {
      // Options
      timeout: 15000,
      enableHighAccuracy: true,
      maximumAge: 300000, // 5 minutes
    },
  );
}

function onKeyDown(e: KeyboardEvent): void {
  switch (e.key) {
    case "Escape":
      document.getElementById("search")?.blur();
      break;

    case "ArrowDown":
      console.log(highlighted.value);
      console.log(visibleElements.value);
      if (visibleElements.value.length === 0) {
        e.preventDefault();
        break;
      }

      highlighted.value = (highlighted.value + 1) % visibleElements.value.length;
      e.preventDefault();
      break;

    case "ArrowUp":
      console.log(highlighted.value);
      console.log(visibleElements.value);
      if (visibleElements.value.length === 0) {
        e.preventDefault();
        break;
      }

      if (highlighted.value <= 0) {
        highlighted.value = visibleElements.value.length - 1;
      } else {
        highlighted.value -= 1;
      }
      e.preventDefault();
      break;

    case "Enter":
      if (highlighted.value !== undefined) {
        e.preventDefault();
        const visible = visibleElements.value[highlighted.value];
        if (visible) {
          select(visible);
        }
      } else {
        query.value = "";
        selected.value = "";
      }
      break;
    default:
      break;
  }
}

const runtimeConfig = useRuntimeConfig();
const url = computed(() => {
  const params = new URLSearchParams();
  params.append("q", query.value);
  params.append("lang", locale.value);
  params.append("pre_highlight", "<b class='text-blue'>");
  params.append("post_highlight", "</b>");
  params.append("search_addresses", "true");

  return `${runtimeConfig.public.apiURL}/api/search?${params.toString()}`;
});
const { data, error } = await useFetch<SearchResponse>(url, {
  dedupe: "cancel",
  credentials: "omit",
  retry: 120,
  retryDelay: 1000,
});
</script>

<template>
  <div
    class="bg-zinc-200 border-zinc-400 flex flex-grow flex-row rounded-s-sm border focus-within:outline focus-within:outline-2 focus-within:outline-offset-1 focus-within:outline-blue-600"
  >
    <textarea
      :id="queryId"
      v-model="query"
      cols="1"
      rows="2"
      :title="t('input.aria-searchlabel')"
      aria-autocomplete="both"
      aria-haspopup="false"
      autocapitalize="off"
      autocomplete="off"
      spellcheck="false"
      maxlength="2048"
      :name="queryId"
      type="text"
      class="text-zinc-800 flex-grow resize-none bg-transparent py-2.5 ps-3 pe-2 text-sm font-semibold placeholder:text-zinc-800 focus-within:placeholder:text-zinc-500 placeholder:font-normal focus:outline-0"
      :placeholder="t('input.placeholder-' + queryId)"
      :aria-label="t('input.aria-searchlabel')"
      @focus="
        console.log('focuseed', queryId);
        currently_actively_picking = true;
        highlighted = 0;
      "
      @keydown="onKeyDown"
    />
    <ClientOnly>
      <button
        v-if="isGeolocationSupported"
        type="button"
        class="focusable text-zinc-600 hover:text-blue-600 hover:bg-blue-50 flex items-center justify-center px-3 py-2.5 transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-transparent rounded-sm text-xs font-medium whitespace-nowrap"
        :disabled="isGettingLocation"
        :title="t('gps.use_current_location')"
        :aria-label="t('gps.use_current_location')"
        @click="useCurrentLocation"
      >
        <MdiIcon
          :path="mdiCrosshairsGps"
          :size="16"
          :class="{
            'animate-pulse text-blue-600': isGettingLocation,
            'text-zinc-600 mr-1': !isGettingLocation,
          }"
        />
      </button>
    </ClientOnly>
  </div>
  <!-- Autocomplete -->
  <ClientOnly>
    <div
      v-if="currently_actively_picking && data && query.length !== 0"
      class="shadow-4xl bg-zinc-50 border-zinc-200 absolute top-3 z-30 -ms-4 mt-56 flex max-h-[calc(100vh-80px)] min-w-96 max-w-sm flex-col gap-4 overflow-auto rounded border p-3.5 shadow-zinc-700/30 md:me-3"
    >
      <Toast v-if="error" id="search-error" level="error">
        <p class="text-md font-bold">{{ t("error.header") }}</p>
        <p class="text-sm">
          {{ t("error.reason") }}:<br />
          <code
            class="text-red-900 bg-red-200 mb-1 mt-2 inline-flex max-w-full items-center space-x-2 overflow-auto rounded-md px-4 py-3 text-left font-mono text-xs dark:bg-red-50/20"
          >
            {{ error }}
          </code>
        </p>
        <p class="text-sm">{{ t("error.call_to_action") }}</p>
      </Toast>
      <ul v-for="s in data.sections" v-cloak :key="s.facet" class="flex flex-col gap-2">
        <div class="flex items-center">
          <span class="text-md text-zinc-800 me-4 flex-shrink">{{ t(`sections.${s.facet}`) }}</span>
          <div class="border-zinc-800 flex-grow border-t" />
        </div>

        <template v-for="(e, i) in s.entries" :key="e.id">
          <SearchResultItem
            v-if="i < s.n_visible"
            :highlighted="e.id === visibleElements[highlighted ?? -1]"
            :item="e"
            @click="select(e.id)"
            @mouseover="highlighted = i"
          />
        </template>
        <li class="-mt-2">
          <Btn
            v-if="s.facet === 'sites_buildings' && !sites_buildings_expanded && s.n_visible < s.entries.length"
            variant="linkButton"
            size="sm"
            @click="sites_buildings_expanded = true"
          >
            {{ t("show_hidden", s.entries.length - s.n_visible) }}
          </Btn>
          <span class="text-zinc-400 text-sm">
            {{
              s.estimatedTotalHits > 20 ? t("approx_results", s.estimatedTotalHits) : t("results", s.estimatedTotalHits)
            }}
          </span>
        </li>
      </ul>
    </div>
  </ClientOnly>
</template>

<i18n lang="yaml">
de:
  input:
    placeholder-from: Von
    placeholder-to: Nach
    aria-actionlabel: Suche nach dem im Suchfeld eingetragenen Raum
    aria-searchlabel: Suchfeld
    action: Go
  show_hidden: +{count} ausgeblendet
  sections:
    sites_buildings: Gebäude / Standorte
    rooms: Räume
    addresses: Adressen
  results: 1 Ergebnis | {count} Ergebnisse
  approx_results: ca. {count} Ergebnisse
  gps:
    use_current_location: Aktuellen Standort verwenden (GPS)
    my_location: Mein Standort
    error:
      permission_denied: Standortzugriff wurde verweigert. Bitte erlaube den Zugriff auf deinen Standort in den Browser-Einstellungen.
      position_unavailable: Standort konnte nicht ermittelt werden. Bitte versuche es später erneut.
      timeout: Standortermittlung dauerte zu lange. Bitte versuche es erneut.
      general: Fehler beim Ermitteln des Standorts. Bitte versuche es erneut.
      not_supported: Geolokation wird von diesem Browser nicht unterstützt.
      https_required: Geolokation erfordert eine sichere Verbindung (HTTPS).
en:
  input:
    placeholder-from: From
    placeholder-to: To
    aria-actionlabel: Search for the room-query entered in the search field
    aria-searchlabel: Search-field
    action: Go
  show_hidden: +{count} hidden
  sections:
    sites_buildings: Buildings / Sites
    rooms: Rooms
    addresses: Adresses
  results: 1 result | {count} results
  approx_results: approx. {count} results
  gps:
    use_current_location: Use current location (GPS)
    my_location: My location
    error:
      permission_denied: Location access was denied. Please allow location access in your browser settings.
      position_unavailable: Location could not be determined. Please try again later.
      timeout: Location request timed out. Please try again.
      general: Error getting location. Please try again.
      not_supported: Geolocation is not supported by this browser.
      https_required: Geolocation requires a secure connection (HTTPS).
</i18n>
