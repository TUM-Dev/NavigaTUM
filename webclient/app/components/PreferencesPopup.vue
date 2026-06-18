<script setup lang="ts">
import { Tab, TabGroup, TabList } from "@headlessui/vue";
import {
  mdiAccountGroup,
  mdiAccountMultiple,
  mdiBike,
  mdiBriefcase,
  mdiBus,
  mdiCar,
  mdiCheck,
  mdiEye,
  mdiImageFilterHdr,
  mdiMonitor,
  mdiMoonWaningCrescent,
  mdiMotorbike,
  mdiRoadVariant,
  mdiSchool,
  mdiSpeedometer,
  mdiTune,
  mdiWalk,
  mdiWheelchairAccessibility,
  mdiWhiteBalanceSunny,
} from "@mdi/js";
import { type EatApiLocale, labelText } from "~/utils/eatApiLabels";
import {
  allergenIcon,
  MENSA_PRICE_ROLES,
  type MensaPriceRole,
  SELECTABLE_ALLERGENS,
} from "~/utils/mensaMenu";

const colorMode = useColorMode();
const { t, locale } = useI18n({ useScope: "local" });
const { preferences, updatePreference } = useUserPreferences();
const { priceRole, allergenWarnings } = useMensaPreferences();
const { isOpen, pendingSection } = usePreferencesPopup();

const labelLocale = computed<EatApiLocale>(() => (locale.value === "de" ? "de" : "en"));

const mensaRoleIcons: Record<MensaPriceRole, string> = {
  students: mdiSchool,
  staff: mdiBriefcase,
  guests: mdiAccountGroup,
};

const switchLocalePath = useSwitchLocalePath();

const allergenSection = ref<HTMLElement | null>(null);

function toggleAllergen(code: string): void {
  const next = new Set(allergenWarnings.value);
  if (next.has(code)) next.delete(code);
  else next.add(code);
  allergenWarnings.value = SELECTABLE_ALLERGENS.filter((allergen) => next.has(allergen));
}

// Deep-linked opens (from the menu) scroll to the section; wait a tick for the lazy modal to mount.
watch(isOpen, async (open) => {
  if (!open || pendingSection.value !== "allergens") return;
  await nextTick();
  allergenSection.value?.scrollIntoView({ behavior: "smooth", block: "start" });
  pendingSection.value = null;
});

watch(locale, async (value) => {
  await updateLocale(value as "de" | "en");
});

async function updateLocale(value: "de" | "en") {
  await navigateTo(switchLocalePath(value));
}
</script>

<template>
  <div>
    <!-- Trigger button; overridable via the `trigger` slot. -->
    <slot name="trigger" :open="() => (isOpen = true)">
      <button
        id="preferences"
        class="focusable relative flex rounded-full bg-transparent p-2 text-sm ring-2 ring-white/0 dark:ring-white/0 hover:bg-zinc-200 dark:hover:bg-zinc-700 hover:ring-white/20 dark:hover:ring-white/20 focus:outline-none focus:ring-white/100 dark:focus:ring-white/100"
        @click="isOpen = true"
      >
        <span class="absolute -inset-1.5" />
        <span class="sr-only">{{ t("open") }}</span>
        <MdiIcon :path="mdiTune" :size="28" class="text-zinc-900 dark:text-zinc-50" />
      </button>
    </slot>

    <!-- Modal Dialog -->
    <ClientOnly>
      <LazyModal v-model="isOpen" :title="t('preferences')" class="bg-white dark:bg-black" @close="isOpen = false">
        <div class="space-y-8">
          <!-- Theme Setting -->
          <div>
            <h3 class="text-lg font-semibold text-zinc-800 dark:text-zinc-100 mb-4">{{ t("theme") }}</h3>
            <TabGroup :default-index="colorMode.preference === 'system' ? 0 : colorMode.preference === 'light' ? 1 : 2">
              <TabList class="flex space-x-1 rounded-lg bg-zinc-100 dark:bg-zinc-800 p-1">
                <Tab as="template" v-slot="{ selected }">
                  <SegmentedTab :selected="selected" class="w-full py-2.5 px-3" @click="colorMode.preference = 'system'">
                    <div class="flex items-center justify-center gap-2">
                      <MdiIcon :path="mdiMonitor" :size="16" />
                      {{ t("theme.system") }}
                    </div>
                  </SegmentedTab>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <SegmentedTab :selected="selected" class="w-full py-2.5 px-3" @click="colorMode.preference = 'light'">
                    <div class="flex items-center justify-center gap-2">
                      <MdiIcon :path="mdiWhiteBalanceSunny" :size="16" />
                      {{ t("theme.light") }}
                    </div>
                  </SegmentedTab>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <SegmentedTab :selected="selected" class="w-full py-2.5 px-3" @click="colorMode.preference = 'dark'">
                    <div class="flex items-center justify-center gap-2">
                      <MdiIcon :path="mdiMoonWaningCrescent" :size="16" />
                      {{ t("theme.dark") }}
                    </div>
                  </SegmentedTab>
                </Tab>
              </TabList>
            </TabGroup>
          </div>

          <!-- Language Setting -->
          <div>
            <h3 class="text-lg font-semibold text-zinc-800 dark:text-zinc-100 mb-4">{{ t("language") }}</h3>
            <TabGroup :default-index="locale === 'de' ? 0 : 1">
              <TabList class="flex space-x-1 rounded-lg bg-zinc-100 dark:bg-zinc-800 p-1">
                <Tab as="template" v-slot="{ selected }">
                  <SegmentedTab :selected="selected" class="w-full py-2.5 px-3" @click="locale = 'de'">
                    Deutsch
                  </SegmentedTab>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <SegmentedTab :selected="selected" class="w-full py-2.5 px-3" @click="locale = 'en'">
                    English
                  </SegmentedTab>
                </Tab>
              </TabList>
            </TabGroup>
          </div>

          <!-- Mensa Price Role Setting -->
          <div>
            <h3 class="text-lg font-semibold text-zinc-800 dark:text-zinc-100 mb-2">{{ t("mensaPriceRole") }}</h3>
            <p class="text-sm text-zinc-600 dark:text-zinc-300 mb-4">{{ t("mensaPriceRole.help") }}</p>
            <TabGroup :default-index="MENSA_PRICE_ROLES.indexOf(priceRole)">
              <TabList class="flex space-x-1 rounded-lg bg-zinc-100 dark:bg-zinc-800 p-1">
                <Tab v-for="role in MENSA_PRICE_ROLES" :key="role" as="template" v-slot="{ selected }">
                  <SegmentedTab :selected="selected" class="w-full py-3 px-4" @click="priceRole = role">
                    <div class="flex items-center justify-center gap-2">
                      <MdiIcon :path="mensaRoleIcons[role]" :size="16" />
                      {{ t(`mensaRole.${role}`) }}
                    </div>
                  </SegmentedTab>
                </Tab>
              </TabList>
            </TabGroup>
          </div>

          <!-- Mensa Allergy Warnings Setting -->
          <div ref="allergenSection">
            <h3 class="text-lg font-semibold text-zinc-800 dark:text-zinc-100 mb-2">{{ t("allergyWarnings") }}</h3>
            <p class="text-sm text-zinc-600 dark:text-zinc-300 mb-4">{{ t("allergyWarnings.help") }}</p>
            <div class="flex flex-wrap gap-2">
              <button
                v-for="code in SELECTABLE_ALLERGENS"
                :key="code"
                type="button"
                :aria-pressed="allergenWarnings.includes(code)"
                class="focusable inline-flex items-center gap-1.5 rounded-full px-3 py-1.5 text-sm font-medium ring-1 transition-colors"
                :class="
                  allergenWarnings.includes(code)
                    ? 'bg-red-100 text-red-800 ring-red-300 dark:bg-red-900/40 dark:text-red-200 dark:ring-red-800'
                    : 'bg-zinc-100 text-zinc-600 ring-transparent hover:bg-zinc-200 dark:bg-zinc-800 dark:text-zinc-300 dark:hover:bg-zinc-700'
                "
                @click="toggleAllergen(code)"
              >
                <MdiIcon :path="allergenIcon(code)" :size="16" class="shrink-0" aria-hidden="true" />
                {{ labelText(code, labelLocale) }}
                <MdiIcon
                  v-if="allergenWarnings.includes(code)"
                  :path="mdiCheck"
                  :size="15"
                  class="shrink-0"
                  aria-hidden="true"
                />
              </button>
            </div>
          </div>

          <!-- Preferred Transport Mode Setting -->
          <div>
            <h3 class="text-lg font-semibold text-zinc-800 dark:text-zinc-100 mb-2">{{ t("preferredTransportMode") }}</h3>
            <p class="text-sm text-zinc-600 dark:text-zinc-300 mb-4">{{ t("preferredTransportMode.help") }}</p>
            <TabGroup :default-index="['pedestrian', 'bicycle', 'motorcycle', 'car', 'public_transit'].indexOf(preferences.route_costing)">
              <TabList class="flex flex-wrap gap-2 rounded-lg bg-zinc-100 dark:bg-zinc-800 p-2">
                <Tab as="template" v-slot="{ selected }">
                  <SegmentedTab :selected="selected" class="px-4 py-3" @click="updatePreference('route_costing', 'pedestrian')">
                    <div class="flex items-center gap-2">
                      <MdiIcon :path="mdiWalk" :size="20" />
                      {{ t("transport.pedestrian") }}
                    </div>
                  </SegmentedTab>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <SegmentedTab :selected="selected" class="px-4 py-3" @click="updatePreference('route_costing', 'bicycle')">
                    <div class="flex items-center gap-2">
                      <MdiIcon :path="mdiBike" :size="20" />
                      {{ t("transport.bicycle") }}
                    </div>
                  </SegmentedTab>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <SegmentedTab :selected="selected" class="px-4 py-3" @click="updatePreference('route_costing', 'motorcycle')">
                    <div class="flex items-center gap-2">
                      <MdiIcon :path="mdiMotorbike" :size="20" />
                      {{ t("transport.motorcycle") }}
                    </div>
                  </SegmentedTab>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <SegmentedTab :selected="selected" class="px-4 py-3" @click="updatePreference('route_costing', 'car')">
                    <div class="flex items-center gap-2">
                      <MdiIcon :path="mdiCar" :size="20" />
                      {{ t("transport.car") }}
                    </div>
                  </SegmentedTab>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <SegmentedTab :selected="selected" class="px-4 py-3" @click="updatePreference('route_costing', 'public_transit')">
                    <div class="flex items-center gap-2">
                      <MdiIcon :path="mdiBus" :size="20" />
                      {{ t("transport.publicTransit") }}
                    </div>
                  </SegmentedTab>
                </Tab>
              </TabList>
            </TabGroup>
          </div>

          <!-- Pedestrian Type Setting -->
          <div>
            <h3 class="text-lg font-semibold text-zinc-800 dark:text-zinc-100 mb-2">{{ t("pedestrianType") }}</h3>
            <p class="text-sm text-zinc-600 dark:text-zinc-300 mb-4">{{ t("pedestrianType.help") }}</p>
            <TabGroup :default-index="preferences.pedestrian_type === 'blind' ? 2 : preferences.pedestrian_type === 'wheelchair' ? 1 : 0">
              <TabList class="flex space-x-1 rounded-lg bg-zinc-100 dark:bg-zinc-800 p-1">
                <Tab as="template" v-slot="{ selected }">
                  <SegmentedTab :selected="selected" class="w-full py-3 px-4" @click="updatePreference('pedestrian_type', 'standard')">
                    <div class="flex items-center justify-center gap-2">
                      <MdiIcon :path="mdiAccountMultiple" :size="16" />
                      {{ t("pedestrian.standard") }}
                    </div>
                  </SegmentedTab>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <SegmentedTab :selected="selected" class="w-full py-3 px-4" @click="updatePreference('pedestrian_type', 'wheelchair')">
                    <div class="flex items-center justify-center gap-2">
                      <MdiIcon :path="mdiWheelchairAccessibility" :size="16" />
                      {{ t("pedestrian.wheelchair") }}
                    </div>
                  </SegmentedTab>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <SegmentedTab :selected="selected" class="w-full py-3 px-4" @click="updatePreference('pedestrian_type', 'blind')">
                    <div class="flex items-center justify-center gap-2">
                      <MdiIcon :path="mdiEye" :size="16" />
                      {{ t("pedestrian.blind") }}
                    </div>
                  </SegmentedTab>
                </Tab>
              </TabList>
            </TabGroup>
          </div>

          <!-- Bicycle Type Setting -->
          <div>
            <h3 class="text-lg font-semibold text-zinc-800 dark:text-zinc-100 mb-2">{{ t("bicycleType") }}</h3>
            <p class="text-sm text-zinc-600 dark:text-zinc-300 mb-4">{{ t("bicycleType.help") }}</p>
            <TabGroup :default-index="['hybrid', 'road', 'cross', 'mountain'].indexOf(preferences.bicycle_type || 'hybrid')">
              <TabList class="grid grid-cols-4 gap-2 rounded-lg bg-zinc-100 dark:bg-zinc-800 p-2">
                <Tab as="template" v-slot="{ selected }">
                  <SegmentedTab :selected="selected" class="px-3 py-3" @click="updatePreference('bicycle_type', 'hybrid')">
                    <div class="flex items-center justify-center gap-2">
                      <MdiIcon :path="mdiBike" :size="16" />
                      {{ t("bicycle.hybrid") }}
                    </div>
                  </SegmentedTab>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <SegmentedTab :selected="selected" class="px-3 py-3" @click="updatePreference('bicycle_type', 'road')">
                    <div class="flex items-center justify-center gap-2">
                      <MdiIcon :path="mdiSpeedometer" :size="16" />
                      {{ t("bicycle.road") }}
                    </div>
                  </SegmentedTab>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <SegmentedTab :selected="selected" class="px-3 py-3" @click="updatePreference('bicycle_type', 'cross')">
                    <div class="flex items-center justify-center gap-2">
                      <MdiIcon :path="mdiRoadVariant" :size="16" />
                      {{ t("bicycle.cross") }}
                    </div>
                  </SegmentedTab>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <SegmentedTab :selected="selected" class="px-3 py-3" @click="updatePreference('bicycle_type', 'mountain')">
                    <div class="flex items-center justify-center gap-2">
                      <MdiIcon :path="mdiImageFilterHdr" :size="16" />
                      {{ t("bicycle.mountain") }}
                    </div>
                  </SegmentedTab>
                </Tab>
              </TabList>
            </TabGroup>
          </div>

          <!-- Motorcycle Type Setting -->
          <div>
            <h3 class="text-lg font-semibold text-zinc-800 dark:text-zinc-100 mb-2">{{ t("ptwType") }}</h3>
            <p class="text-sm text-zinc-600 dark:text-zinc-300 mb-4">{{ t("ptwType.help") }}</p>
            <TabGroup :default-index="['motorcycle', 'moped'].indexOf(preferences.ptw_type || 'motorcycle')">
              <TabList class="flex space-x-1 rounded-lg bg-zinc-100 dark:bg-zinc-800 p-1">
                <Tab as="template" v-slot="{ selected }">
                  <SegmentedTab :selected="selected" class="w-full py-3 px-4" @click="updatePreference('ptw_type', 'motorcycle')">
                    <div class="flex items-center justify-center gap-2">
                      <MdiIcon :path="mdiMotorbike" :size="16" />
                      {{ t("ptw.motorcycle") }}
                    </div>
                  </SegmentedTab>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <SegmentedTab :selected="selected" class="w-full py-3 px-4" @click="updatePreference('ptw_type', 'moped')">
                    <div class="flex items-center justify-center gap-2">
                      <MdiIcon :path="mdiBike" :size="16" />
                      {{ t("ptw.moped") }}
                    </div>
                  </SegmentedTab>
                </Tab>
              </TabList>
            </TabGroup>
          </div>
        </div>
      </LazyModal>
    </ClientOnly>
  </div>
</template>

<i18n lang="yaml">
de:
  preferences: Präferenzen
  open: Einstellungsmenü öffnen
  language: Sprache
  theme: Design
  theme.system: System
  theme.dark: Dunkel
  theme.light: Hell
  mensaPriceRole: Mensa-Preisgruppe
  mensaPriceRole.help: Bestimmt, welcher Preis im Speiseplan hervorgehoben wird.
  mensaRole.students: Studierende
  mensaRole.staff: Bedienstete
  mensaRole.guests: Gäste
  allergyWarnings: Allergiewarnungen
  allergyWarnings.help: Gerichte mit den ausgewählten Allergenen werden im Speiseplan rot hervorgehoben.
  preferredTransportMode: Bevorzugtes Verkehrsmittel
  preferredTransportMode.help: Dies wird als Standard für die Navigation verwendet.
  transport.pedestrian: Zu Fuß
  transport.bicycle: Fahrrad
  transport.motorcycle: Motorrad
  transport.car: Auto
  transport.publicTransit: Öffentliche Verkehrsmittel
  pedestrianType: Fußgänger-Typ
  pedestrianType.help: Wählen Sie dies, falls Sie Barrierefreiheit benötigen wie Ansagen oder Aufzüge.
  pedestrian.standard: Standard
  pedestrian.blind: Blind
  pedestrian.wheelchair: Rollstuhl
  bicycleType: Fahrrad-Typ
  bicycleType.help: Dies beeinflusst welche Wege für Sie ausgewählt werden. Rennräder meiden unbefestigte Wege.
  bicycle.road: Rennrad
  bicycle.hybrid: Standard
  bicycle.cross: Crossrad
  bicycle.mountain: Mountainbike
  ptwType: Zweirad-Typ
  ptwType.help: Dies beeinflusst welche Straßen Sie befahren dürfen und Ihre Geschwindigkeitsbegrenzungen.
  ptw.motorcycle: Motorrad
  ptw.moped: Moped
en:
  preferences: Preferences
  open: Open preferences menu
  language: Language
  theme: Theme
  theme.system: System
  theme.dark: Dark
  theme.light: Light
  mensaPriceRole: Canteen price group
  mensaPriceRole.help: Determines which price is highlighted in the menu.
  mensaRole.students: Students
  mensaRole.staff: Staff
  mensaRole.guests: Guests
  allergyWarnings: Allergy warnings
  allergyWarnings.help: Dishes containing the selected allergens are highlighted in red in the menu.
  preferredTransportMode: Preferred Transport Mode
  preferredTransportMode.help: This will be used as the default for navigation.
  transport.pedestrian: Walking
  transport.bicycle: Bicycle
  transport.motorcycle: Motorcycle
  transport.car: Car
  transport.publicTransit: Transit
  pedestrianType: Pedestrian Type
  pedestrianType.help: Select this if you need accessibility features like narration or elevators.
  pedestrian.standard: Standard
  pedestrian.blind: Blind
  pedestrian.wheelchair: Wheelchair
  bicycleType: Bicycle Type
  bicycleType.help: This affects which paths are selected for you. Road bikes avoid unpaved paths.
  bicycle.road: Road
  bicycle.hybrid: Standard
  bicycle.cross: Cross
  bicycle.mountain: Mountain
  ptwType: Two-Wheeler Type
  ptwType.help: This affects which roads you can use and your speed limits.
  ptw.motorcycle: Motorcycle
  ptw.moped: Moped
</i18n>
