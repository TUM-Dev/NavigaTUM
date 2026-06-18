<script setup lang="ts">
import { mdiAlert, mdiBellRingOutline, mdiInformationOutline } from "@mdi/js";
import type { components } from "~/api_types";
import { type EatApiLocale, labelText } from "~/utils/eatApiLabels";
import {
  allergenLabels,
  dietMarker,
  formatEuro,
  groupAllergensByIcon,
  type MensaPriceRole,
  matchedAllergens,
} from "~/utils/mensaMenu";

type MenuDish = components["schemas"]["MensaMenuDishResponse"];
type MenuPrice = components["schemas"]["MensaMenuPriceResponse"];

const props = defineProps<{
  readonly dish: MenuDish;
  readonly priceRole: MensaPriceRole;
}>();

const { t, locale } = useI18n({ useScope: "local" });
const { allergenWarnings } = useMensaPreferences();
const { open: openPreferences } = usePreferencesPopup();

const labelLocale = computed<EatApiLocale>(() => (locale.value === "de" ? "de" : "en"));
const diet = computed(() => dietMarker(props.dish.labels));
const allergens = computed(() => allergenLabels(props.dish.labels));
const allergenText = computed(() =>
  allergens.value.map((code) => labelText(code, labelLocale.value)).join(", ")
);
const allergenRows = computed(() =>
  groupAllergensByIcon(allergens.value).map((group) => ({
    icon: group.icon,
    text: group.codes.map((code) => labelText(code, labelLocale.value)).join(", "),
  }))
);
// Matched against the full label set so a diet-promoted label such as `fish` still warns.
const warnings = computed(() => matchedAllergens(props.dish.labels, allergenWarnings.value));
const warningText = computed(() =>
  warnings.value.map((code) => labelText(code, labelLocale.value)).join(", ")
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
  <li
    class="flex flex-col gap-1.5"
    :class="
      warnings.length > 0 &&
      'border-red-300 bg-red-50 dark:border-red-900/70 dark:bg-red-950/40 -mx-2 rounded-sm border p-2'
    "
  >
    <div class="flex flex-row items-baseline justify-between gap-3">
      <p
        class="text-sm font-medium"
        :class="
          warnings.length ? 'text-red-800 dark:text-red-200' : 'text-zinc-800 dark:text-zinc-100'
        "
      >
        {{ dish.name }}
      </p>
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
        <!-- No `mt` gap: the panel must abut the button so the pointer can reach the link inside. -->
        <span
          class="ring-black/5 dark:ring-white/10 absolute left-0 top-full z-20 hidden w-max max-w-64 flex-col gap-2 rounded bg-white p-2.5 pt-2 text-left text-sm leading-snug text-zinc-700 shadow-lg ring-1 group-hover:flex group-focus-within:flex dark:bg-zinc-800 dark:text-zinc-200"
        >
          <span class="flex flex-col gap-1">
            <span class="text-zinc-500 dark:text-zinc-400 text-xs font-semibold uppercase tracking-wide">
              {{ t("contains") }}
            </span>
            <span
              v-for="row in allergenRows"
              :key="row.icon"
              class="flex items-start gap-1.5"
            >
              <MdiIcon
                :path="row.icon"
                :size="15"
                class="text-zinc-400 dark:text-zinc-500 mt-0.5 shrink-0"
                aria-hidden="true"
              />
              <span>{{ row.text }}</span>
            </span>
          </span>
          <Btn variant="link" size="text-xs rounded" @click="openPreferences('allergens')">
            <MdiIcon :path="mdiBellRingOutline" :size="13" class="mt-0.5 shrink-0" aria-hidden="true" />
            <span>{{ t("configure_warnings") }}</span>
          </Btn>
        </span>
      </span>
    </div>

    <p
      v-if="warnings.length"
      class="text-red-700 dark:text-red-300 flex items-start gap-1.5 text-xs font-medium"
    >
      <MdiIcon :path="mdiAlert" :size="14" class="mt-0.5 shrink-0" aria-hidden="true" />
      <span>{{ t("allergy_warning", { allergens: warningText }) }}</span>
    </p>
  </li>
</template>

<i18n lang="yaml">
de:
  allergens: "Allergene & Zusatzstoffe ({count})"
  allergy_warning: "Achtung, enthält {allergens}"
  contains: "Enthält:"
  configure_warnings: Lass dich vor Allergenen warnen, die dich betreffen
  price_with_unit: "{base} + {perUnit}/{unit}"
  price_per_unit_only: "{perUnit}/{unit}"
en:
  allergens: "Allergens & additives ({count})"
  allergy_warning: "Heads-up: contains {allergens}"
  contains: "Contains:"
  configure_warnings: Let us warn you about allergens you care about
  price_with_unit: "{base} + {perUnit}/{unit}"
  price_per_unit_only: "{perUnit}/{unit}"
</i18n>
