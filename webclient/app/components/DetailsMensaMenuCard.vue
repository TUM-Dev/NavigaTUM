<script setup lang="ts">
import { mdiChevronDown, mdiOpenInNew, mdiSilverwareForkKnife } from "@mdi/js";
import { useToggle } from "@vueuse/core";
import type { components } from "~/api_types";
import { type EatApiLocale, labelText } from "~/utils/eatApiLabels";

type MenuResponse = components["schemas"]["MensaMenuResponse"];
type MenuDay = components["schemas"]["MensaMenuDayResponse"];
type MenuDish = components["schemas"]["MensaMenuDishResponse"];
type MenuPrice = components["schemas"]["MensaMenuPriceResponse"];
type RoleKey = keyof MenuDish["prices"];

const props = defineProps<{
  readonly slug: string;
}>();

const { t, locale } = useI18n({ useScope: "local" });
const runtimeConfig = useRuntimeConfig();
const [expanded, toggleExpanded] = useToggle(false);

// `server: false` keeps the live menu out of the page's 1h SWR detail HTML so it is always fresh.
const { data: menu, status } = await useFetch<MenuResponse>(
  () => `${runtimeConfig.public.apiURL}/api/mensa/${encodeURIComponent(props.slug)}`,
  { server: false, lazy: true, credentials: "omit" }
);
// `idle` is the pre-request tick; folding it into loading avoids a flash of the empty-menu box.
const isLoading = computed<boolean>(() => status.value === "idle" || status.value === "pending");
const hasError = computed<boolean>(() => status.value === "error");
const days = computed<readonly MenuDay[]>(() => menu.value?.days ?? []);

// The eat-api `date` is a plain `YYYY-MM-DD` string in Munich's local time, so we compare
// against the visitor's local day rather than walking a Date through UTC.
const todayIso = computed(() => {
  const now = new Date();
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`;
});

const todayDay = computed<MenuDay | null>(
  () => days.value.find((day) => day.date === todayIso.value) ?? null
);

// Today renders inline above the chevron, so the expanded block lists everything after.
const futureDays = computed<readonly MenuDay[]>(() =>
  days.value.filter((day) => day.date > todayIso.value)
);

// First future day surfaces in the collapsed view when today is closed; visitors should not
// have to expand to find out what tomorrow looks like.
const headlineDay = computed<MenuDay | null>(() => todayDay.value ?? futureDays.value[0] ?? null);
const headlineIsToday = computed<boolean>(() => headlineDay.value === todayDay.value);

// When today _is_ the headline, the expandable block holds the remaining days; when today is
// closed and tomorrow is the headline, we skip showing it again in the expanded block.
const remainingDays = computed<readonly MenuDay[]>(() =>
  headlineIsToday.value ? futureDays.value : futureDays.value.slice(1)
);

const labelLocale = computed<EatApiLocale>(() => (locale.value === "de" ? "de" : "en"));

// Roles render in a fixed order so dishes compare visually across rows.
const ROLES = ["students", "staff", "guests"] as const satisfies readonly RoleKey[];

const euroFormatter = computed(
  () =>
    new Intl.NumberFormat(locale.value === "de" ? "de-DE" : "en-GB", {
      style: "currency",
      currency: "EUR",
    })
);

function formatPrice(price: MenuPrice): string {
  const base = euroFormatter.value.format(price.base_price);
  if (price.price_per_unit && price.unit) {
    // Per-unit dishes (e.g. 100g self-service) carry a non-zero `price_per_unit`; render both
    // so visitors see the headline number and the per-unit charge that drives the final total.
    const perUnit = euroFormatter.value.format(price.price_per_unit);
    return t("price_with_unit", { base, perUnit, unit: price.unit });
  }
  return base;
}

function formatDayLabel(iso: string): string {
  const [year, month, day] = iso.split("-").map(Number);
  if (!year || !month || !day) return iso;
  const date = new Date(year, month - 1, day);
  return date.toLocaleDateString(locale.value === "de" ? "de-DE" : "en-GB", {
    weekday: "long",
    day: "numeric",
    month: "long",
  });
}

const lastUpdated = computed(() => {
  const raw = menu.value?.last_update;
  if (!raw) return "";
  const [year, month, day] = raw.split("-").map(Number);
  if (!year || !month || !day) return raw;
  return new Date(year, month - 1, day).toLocaleDateString(
    locale.value === "de" ? "de-DE" : "en-GB",
    {
      year: "numeric",
      month: "long",
      day: "numeric",
    }
  );
});
</script>

<template>
  <section class="flex flex-col gap-3 print:!hidden">
    <div class="flex flex-row items-baseline justify-between gap-2">
      <p class="text-zinc-800 dark:text-zinc-100 text-lg font-semibold">{{ t("title") }}</p>
      <Btn v-if="menu" :to="menu.source_url" variant="link" size="text-xs gap-1 rounded">
        {{ t("source") }}
        <MdiIcon :path="mdiOpenInNew" :size="14" class="my-auto" aria-hidden="true" />
      </Btn>
    </div>

    <p
      v-if="isLoading"
      class="bg-zinc-100 dark:bg-zinc-800 border-zinc-200 dark:border-zinc-700 text-zinc-500 dark:text-zinc-400 animate-pulse rounded-sm border p-3 text-sm"
      aria-live="polite"
    >
      {{ t("loading") }}
    </p>
    <p
      v-else-if="hasError"
      class="bg-zinc-100 dark:bg-zinc-800 border-zinc-200 dark:border-zinc-700 text-zinc-500 dark:text-zinc-400 rounded-sm border p-3 text-sm"
      role="alert"
    >
      {{ t("error") }}
    </p>
    <div
      v-else
      class="bg-zinc-100 dark:bg-zinc-800 border-zinc-200 dark:border-zinc-700 rounded-sm border"
    >
      <div v-if="headlineDay" class="border-zinc-200 dark:border-zinc-700 border-b p-3">
        <p class="text-zinc-800 dark:text-zinc-100 mb-2 flex items-center gap-1.5 font-medium">
          <MdiIcon :path="mdiSilverwareForkKnife" :size="16" class="shrink-0" aria-hidden="true" />
          <span>
            {{ headlineIsToday ? t("today") : t("next_open", { day: formatDayLabel(headlineDay.date) }) }}
          </span>
        </p>
        <ul class="flex flex-col gap-2.5">
          <li v-for="(dish, index) in headlineDay.dishes" :key="index" class="flex flex-col gap-1">
            <div class="flex flex-row items-baseline justify-between gap-3">
              <p class="text-zinc-800 dark:text-zinc-100 text-sm font-medium">{{ dish.name }}</p>
              <p v-if="dish.dish_type" class="text-zinc-500 dark:text-zinc-400 shrink-0 text-xs">
                {{ dish.dish_type }}
              </p>
            </div>
            <ul class="text-zinc-600 dark:text-zinc-300 flex flex-wrap gap-x-3 gap-y-0.5 text-xs">
              <template v-for="role in ROLES" :key="role">
                <li v-if="dish.prices[role]">
                  <span class="text-zinc-500 dark:text-zinc-400">{{ t(`role.${role}`) }}:</span>
                  {{ formatPrice(dish.prices[role]!) }}
                </li>
              </template>
            </ul>
            <ul v-if="dish.labels.length" class="flex flex-wrap gap-1">
              <li
                v-for="code in dish.labels"
                :key="code"
                :title="labelText(code, labelLocale)"
                class="bg-zinc-200 dark:bg-zinc-700 text-zinc-700 dark:text-zinc-200 rounded px-1.5 py-0.5 text-[10px] tracking-wide uppercase"
              >
                {{ labelText(code, labelLocale) }}
              </li>
            </ul>
          </li>
        </ul>
      </div>
      <p v-else class="text-zinc-500 dark:text-zinc-400 p-3 text-sm">{{ t("no_menu") }}</p>

      <button
        v-if="remainingDays.length"
        type="button"
        class="focusable flex w-full items-center gap-3 p-3 text-left"
        :aria-expanded="expanded"
        @click="toggleExpanded()"
      >
        <span class="text-zinc-800 dark:text-zinc-100 font-medium">
          {{ expanded ? t("hide_week") : t("show_week", remainingDays.length) }}
        </span>
        <span class="flex-1" />
        <MdiIcon
          :path="mdiChevronDown"
          :size="18"
          class="text-zinc-500 dark:text-zinc-400 shrink-0 transition-transform"
          :class="{ 'rotate-180': expanded }"
          aria-hidden="true"
        />
      </button>
      <div
        v-if="expanded"
        class="border-zinc-200 dark:border-zinc-700 flex flex-col gap-4 border-t p-3"
      >
        <div v-for="day in remainingDays" :key="day.date" class="flex flex-col gap-2.5">
          <p class="text-zinc-700 dark:text-zinc-200 font-medium">{{ formatDayLabel(day.date) }}</p>
          <ul class="flex flex-col gap-2.5">
            <li v-for="(dish, index) in day.dishes" :key="index" class="flex flex-col gap-1">
              <div class="flex flex-row items-baseline justify-between gap-3">
                <p class="text-zinc-800 dark:text-zinc-100 text-sm font-medium">{{ dish.name }}</p>
                <p v-if="dish.dish_type" class="text-zinc-500 dark:text-zinc-400 shrink-0 text-xs">
                  {{ dish.dish_type }}
                </p>
              </div>
              <ul class="text-zinc-600 dark:text-zinc-300 flex flex-wrap gap-x-3 gap-y-0.5 text-xs">
                <template v-for="role in ROLES" :key="role">
                  <li v-if="dish.prices[role]">
                    <span class="text-zinc-500 dark:text-zinc-400">{{ t(`role.${role}`) }}:</span>
                    {{ formatPrice(dish.prices[role]!) }}
                  </li>
                </template>
              </ul>
              <ul v-if="dish.labels.length" class="flex flex-wrap gap-1">
                <li
                  v-for="code in dish.labels"
                  :key="code"
                  :title="labelText(code, labelLocale)"
                  class="bg-zinc-200 dark:bg-zinc-700 text-zinc-700 dark:text-zinc-200 rounded px-1.5 py-0.5 text-[10px] tracking-wide uppercase"
                >
                  {{ labelText(code, labelLocale) }}
                </li>
              </ul>
            </li>
          </ul>
        </div>
      </div>
    </div>

    <small v-if="menu" class="text-zinc-500 dark:text-zinc-400">
      {{ t("last_updated", { date: lastUpdated }) }}
    </small>
  </section>
</template>

<i18n lang="yaml">
de:
  title: Speisekarte
  source: Quelle
  today: Heute
  next_open: "Nächste Öffnung: {day}"
  loading: Speiseplan wird geladen …
  error: Speiseplan konnte nicht geladen werden.
  no_menu: Diese Woche kein Speiseplan online.
  show_week: "noch ein weiterer Tag | noch {count} weitere Tage"
  hide_week: Weitere Tage ausblenden
  last_updated: "zuletzt aktualisiert am {date}"
  price_with_unit: "{base} + {perUnit}/{unit}"
  role:
    students: Studierende
    staff: Bedienstete
    guests: Gäste
en:
  title: Menu
  source: Source
  today: Today
  next_open: "Next opening: {day}"
  loading: Loading menu …
  error: The menu could not be loaded.
  no_menu: No menu published for this week.
  show_week: "one more day | {count} more days"
  hide_week: Hide additional days
  last_updated: "last updated on {date}"
  price_with_unit: "{base} + {perUnit}/{unit}"
  role:
    students: Students
    staff: Staff
    guests: Guests
</i18n>
