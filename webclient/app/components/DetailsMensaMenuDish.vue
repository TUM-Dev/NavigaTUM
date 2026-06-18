<script setup lang="ts">
import { mdiInformationOutline } from "@mdi/js";
import type { components } from "~/api_types";
import { type EatApiLocale, labelText } from "~/utils/eatApiLabels";
import { allergenLabels, dietMarker, formatEuro, type MensaPriceRole } from "~/utils/mensaMenu";

type MenuDish = components["schemas"]["MensaMenuDishResponse"];
type MenuPrice = components["schemas"]["MensaMenuPriceResponse"];

const props = defineProps<{
  readonly dish: MenuDish;
  readonly priceRole: MensaPriceRole;
}>();

const { t, locale } = useI18n({ useScope: "local" });

const labelLocale = computed<EatApiLocale>(() => (locale.value === "de" ? "de" : "en"));
const diet = computed(() => dietMarker(props.dish.labels));
const allergens = computed(() => allergenLabels(props.dish.labels));
const allergenText = computed(() =>
  allergens.value.map((code) => labelText(code, labelLocale.value)).join(", ")
);
const price = computed<MenuPrice | null>(() => props.dish.prices[props.priceRole] ?? null);

// Vegan/vegetarian both read as a positive but need distinct shades so they are told apart at a
// glance: vegan is a deep emerald, vegetarian a lighter lime. Fish/meat are neutral wayfinding.
const DIET_CLASSES: Record<string, string> = {
  vegan: "bg-emerald-100 text-emerald-800 dark:bg-emerald-900/40 dark:text-emerald-200",
  vegetarian: "bg-lime-100 text-lime-800 dark:bg-lime-900/40 dark:text-lime-200",
  fish: "bg-sky-100 text-sky-800 dark:bg-sky-900/40 dark:text-sky-200",
  meat: "bg-orange-100 text-orange-800 dark:bg-orange-900/40 dark:text-orange-200",
};

function formatPrice(value: MenuPrice): string {
  const loc = locale.value === "de" ? "de" : "en";
  if (value.price_per_unit && value.unit) {
    const perUnit = formatEuro(value.price_per_unit, loc);
    // Self-service dishes priced purely by weight carry a `0` base; the per-unit rate is the whole
    // price, so drop the misleading "0,00 € +" prefix.
    if (!value.base_price) return t("price_per_unit_only", { perUnit, unit: value.unit });
    return t("price_with_unit", {
      base: formatEuro(value.base_price, loc),
      perUnit,
      unit: value.unit,
    });
  }
  return formatEuro(value.base_price, loc);
}
</script>

<template>
  <li class="flex flex-col gap-1.5">
    <div class="flex flex-row items-baseline justify-between gap-3">
      <p class="text-zinc-800 dark:text-zinc-100 text-sm font-medium">{{ dish.name }}</p>
      <p
        v-if="price"
        class="text-zinc-800 dark:text-zinc-100 shrink-0 text-sm font-semibold tabular-nums"
      >
        {{ formatPrice(price) }}
      </p>
    </div>

    <div v-if="diet || allergens.length" class="flex flex-wrap items-center gap-x-2 gap-y-1">
      <span
        v-if="diet"
        :class="DIET_CLASSES[diet.kind]"
        class="inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-medium"
      >
        <MdiIcon :path="diet.icon" :size="13" class="shrink-0" aria-hidden="true" />
        {{ labelText(diet.labelCode, labelLocale) }}
      </span>
      <span v-if="allergens.length" class="group relative inline-flex">
        <button
          type="button"
          class="focusable text-zinc-500 dark:text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-200 inline-flex cursor-help items-center gap-1 rounded text-xs"
          :aria-label="`${t('allergens', { count: allergens.length })}: ${allergenText}`"
        >
          <MdiIcon :path="mdiInformationOutline" :size="13" class="shrink-0" aria-hidden="true" />
          {{ t("allergens", { count: allergens.length }) }}
        </button>
        <span
          aria-hidden="true"
          class="ring-black/5 dark:ring-white/10 pointer-events-none absolute left-0 top-full z-20 mt-1 hidden w-max max-w-60 rounded bg-white px-2.5 py-1.5 text-left text-sm leading-snug text-zinc-700 shadow-lg ring-1 group-hover:block group-focus-within:block dark:bg-zinc-800 dark:text-zinc-200"
        >
          {{ allergenText }}
        </span>
      </span>
    </div>
  </li>
</template>

<i18n lang="yaml">
de:
  allergens: "Allergene & Zusatzstoffe ({count})"
  price_with_unit: "{base} + {perUnit}/{unit}"
  price_per_unit_only: "{perUnit}/{unit}"
en:
  allergens: "Allergens & additives ({count})"
  price_with_unit: "{base} + {perUnit}/{unit}"
  price_per_unit_only: "{perUnit}/{unit}"
</i18n>
