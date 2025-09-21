<script setup lang="ts">
import { Tab, TabGroup, TabList } from "@headlessui/vue";
import { mdiMonitor, mdiMoonWaningCrescent, mdiTune, mdiWhiteBalanceSunny } from "@mdi/js";
import { mdiWalk, mdiBike, mdiScooter, mdiCar, mdiBus } from "@mdi/js";
import Modal from "~/components/Modal.vue";
import type { UserRoutingPreferences } from "~/composables/userPreferences";

const colorMode = useColorMode();
const { t } = useI18n({ useScope: "local" });
const { preferences, updatePreference } = useUserPreferences();

const { locale } = useI18n();
const switchLocalePath = useSwitchLocalePath();

const isOpen = ref(false);

watch(locale, async (value) => {
  await updateLocale(value as "de" | "en");
});

async function updateLocale(value: "de" | "en") {
  await navigateTo(switchLocalePath(value));
}

function openPreferences() {
  isOpen.value = true;
}

function closePreferences() {
  isOpen.value = false;
}

// Setting update functions
function selectTransportMode(mode: UserRoutingPreferences["route_costing"]) {
  updatePreference("route_costing", mode);
}

function selectPedestrianType(type: "none" | "blind") {
  updatePreference("pedestrian_type", type);
}

function selectBicycleType(type: NonNullable<UserRoutingPreferences["bicycle_type"]>) {
  updatePreference("bicycle_type", type);
}

function selectPtwType(type: NonNullable<UserRoutingPreferences["ptw_type"]>) {
  updatePreference("ptw_type", type);
}

function selectTheme(theme: "system" | "dark" | "light") {
  colorMode.preference = theme;
}

function selectLanguage(lang: "de" | "en") {
  locale.value = lang;
}
</script>

<template>
  <div>
    <!-- Trigger Button -->
    <button
      id="preferences"
      class="focusable relative flex rounded-full bg-transparent p-2 text-sm ring-2 ring-white ring-opacity-0 hover:bg-zinc-100/10 hover:ring-opacity-20 focus:outline-none focus:ring-opacity-100"
      @click="openPreferences"
    >
      <span class="absolute -inset-1.5" />
      <span class="sr-only">Open preferences menu</span>
      <MdiIcon :path="mdiTune" :size="28" class="text-zinc-900" />
    </button>

    <!-- Modal Dialog -->
    <ClientOnly>
      <LazyModal v-model="isOpen" :title="t('preferences')" class="bg-white" @close="closePreferences">
        <div class="space-y-8">
          <!-- Theme Setting -->
          <div>
            <h3 class="text-lg font-semibold text-zinc-800 mb-4">{{ t("theme") }}</h3>
            <TabGroup :default-index="colorMode.preference === 'system' ? 0 : colorMode.preference === 'light' ? 1 : 2">
              <TabList class="flex space-x-1 rounded-lg bg-zinc-100 p-1">
                <Tab as="template" v-slot="{ selected }">
                  <button
                    :class="[
                      'w-full rounded-md py-2.5 px-3 text-sm font-medium leading-5',
                      'ring-white ring-opacity-60 ring-offset-2 ring-offset-blue-400',
                      'focus:outline-none focus:ring-2 transition-all',
                      selected
                        ? 'bg-white text-zinc-700 shadow'
                        : 'text-zinc-500 hover:bg-white/[0.12] hover:text-zinc-700',
                    ]"
                    @click="selectTheme('system')"
                  >
                    <div class="flex items-center justify-center gap-2">
                      <MdiIcon :path="mdiMonitor" :size="16" />
                      {{ t("theme.system") }}
                    </div>
                  </button>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <button
                    :class="[
                      'w-full rounded-md py-2.5 px-3 text-sm font-medium leading-5',
                      'ring-white ring-opacity-60 ring-offset-2 ring-offset-blue-400',
                      'focus:outline-none focus:ring-2 transition-all',
                      selected
                        ? 'bg-white text-zinc-700 shadow'
                        : 'text-zinc-500 hover:bg-white/[0.12] hover:text-zinc-700',
                    ]"
                    @click="selectTheme('light')"
                  >
                    <div class="flex items-center justify-center gap-2">
                      <MdiIcon :path="mdiWhiteBalanceSunny" :size="16" />
                      {{ t("theme.light") }}
                    </div>
                  </button>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <button
                    :class="[
                      'w-full rounded-md py-2.5 px-3 text-sm font-medium leading-5',
                      'ring-white ring-opacity-60 ring-offset-2 ring-offset-blue-400',
                      'focus:outline-none focus:ring-2 transition-all',
                      selected
                        ? 'bg-white text-zinc-700 shadow'
                        : 'text-zinc-500 hover:bg-white/[0.12] hover:text-zinc-700',
                    ]"
                    @click="selectTheme('dark')"
                  >
                    <div class="flex items-center justify-center gap-2">
                      <MdiIcon :path="mdiMoonWaningCrescent" :size="16" />
                      {{ t("theme.dark") }}
                    </div>
                  </button>
                </Tab>
              </TabList>
            </TabGroup>
          </div>

          <!-- Language Setting -->
          <div>
            <h3 class="text-lg font-semibold text-zinc-800 mb-4">{{ t("language") }}</h3>
            <TabGroup :default-index="locale === 'de' ? 0 : 1">
              <TabList class="flex space-x-1 rounded-lg bg-zinc-100 p-1">
                <Tab as="template" v-slot="{ selected }">
                  <button
                    :class="[
                      'w-full rounded-md py-2.5 px-3 text-sm font-medium leading-5',
                      'ring-white ring-opacity-60 ring-offset-2 ring-offset-blue-400',
                      'focus:outline-none focus:ring-2 transition-all',
                      selected
                        ? 'bg-white text-zinc-700 shadow'
                        : 'text-zinc-500 hover:bg-white/[0.12] hover:text-zinc-700',
                    ]"
                    @click="selectLanguage('de')"
                  >
                    Deutsch
                  </button>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <button
                    :class="[
                      'w-full rounded-md py-2.5 px-3 text-sm font-medium leading-5',
                      'ring-white ring-opacity-60 ring-offset-2 ring-offset-blue-400',
                      'focus:outline-none focus:ring-2 transition-all',
                      selected
                        ? 'bg-white text-zinc-700 shadow'
                        : 'text-zinc-500 hover:bg-white/[0.12] hover:text-zinc-700',
                    ]"
                    @click="selectLanguage('en')"
                  >
                    English
                  </button>
                </Tab>
              </TabList>
            </TabGroup>
          </div>

          <!-- Preferred Transport Mode Setting -->
          <div>
            <h3 class="text-lg font-semibold text-zinc-800 mb-2">{{ t("preferredTransportMode") }}</h3>
            <p class="text-sm text-zinc-600 mb-4">{{ t("preferredTransportMode.help") }}</p>
            <TabGroup
              :default-index="
                ['pedestrian', 'bicycle', 'motorcycle', 'car', 'public_transit'].indexOf(preferences.route_costing)
              "
            >
              <TabList class="flex flex-wrap gap-2 rounded-lg bg-zinc-100 p-2">
                <Tab as="template" v-slot="{ selected }">
                  <button
                    :class="[
                      'rounded-md px-4 py-3 text-sm font-medium leading-5',
                      'ring-white ring-opacity-60 ring-offset-2 ring-offset-blue-400',
                      'focus:outline-none focus:ring-2 transition-all',
                      selected
                        ? 'bg-white text-zinc-700 shadow'
                        : 'text-zinc-500 hover:bg-white/[0.12] hover:text-zinc-700',
                    ]"
                    @click="selectTransportMode('pedestrian')"
                  >
                    <div class="flex items-center gap-2">
                      <MdiIcon :path="mdiWalk" :size="20" />
                      {{ t("transport.pedestrian") }}
                    </div>
                  </button>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <button
                    :class="[
                      'rounded-md px-4 py-3 text-sm font-medium leading-5',
                      'ring-white ring-opacity-60 ring-offset-2 ring-offset-blue-400',
                      'focus:outline-none focus:ring-2 transition-all',
                      selected
                        ? 'bg-white text-zinc-700 shadow'
                        : 'text-zinc-500 hover:bg-white/[0.12] hover:text-zinc-700',
                    ]"
                    @click="selectTransportMode('bicycle')"
                  >
                    <div class="flex items-center gap-2">
                      <MdiIcon :path="mdiBike" :size="20" />
                      {{ t("transport.bicycle") }}
                    </div>
                  </button>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <button
                    :class="[
                      'rounded-md px-4 py-3 text-sm font-medium leading-5',
                      'ring-white ring-opacity-60 ring-offset-2 ring-offset-blue-400',
                      'focus:outline-none focus:ring-2 transition-all',
                      selected
                        ? 'bg-white text-zinc-700 shadow'
                        : 'text-zinc-500 hover:bg-white/[0.12] hover:text-zinc-700',
                    ]"
                    @click="selectTransportMode('motorcycle')"
                  >
                    <div class="flex items-center gap-2">
                      <MdiIcon :path="mdiScooter" :size="20" />
                      {{ t("transport.motorcycle") }}
                    </div>
                  </button>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <button
                    :class="[
                      'rounded-md px-4 py-3 text-sm font-medium leading-5',
                      'ring-white ring-opacity-60 ring-offset-2 ring-offset-blue-400',
                      'focus:outline-none focus:ring-2 transition-all',
                      selected
                        ? 'bg-white text-zinc-700 shadow'
                        : 'text-zinc-500 hover:bg-white/[0.12] hover:text-zinc-700',
                    ]"
                    @click="selectTransportMode('car')"
                  >
                    <div class="flex items-center gap-2">
                      <MdiIcon :path="mdiCar" :size="20" />
                      {{ t("transport.car") }}
                    </div>
                  </button>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <button
                    :class="[
                      'rounded-md px-4 py-3 text-sm font-medium leading-5',
                      'ring-white ring-opacity-60 ring-offset-2 ring-offset-blue-400',
                      'focus:outline-none focus:ring-2 transition-all',
                      selected
                        ? 'bg-white text-zinc-700 shadow'
                        : 'text-zinc-500 hover:bg-white/[0.12] hover:text-zinc-700',
                    ]"
                    @click="selectTransportMode('public_transit')"
                  >
                    <div class="flex items-center gap-2">
                      <MdiIcon :path="mdiBus" :size="20" />
                      {{ t("transport.publicTransit") }}
                    </div>
                  </button>
                </Tab>
              </TabList>
            </TabGroup>
          </div>

          <!-- Pedestrian Type Setting -->
          <div>
            <h3 class="text-lg font-semibold text-zinc-800 mb-2">{{ t("pedestrianType") }}</h3>
            <p class="text-sm text-zinc-600 mb-4">{{ t("pedestrianType.help") }}</p>
            <TabGroup :default-index="['none', 'blind', 'wheelchair'].indexOf(preferences.pedestrian_type || 'none')">
              <TabList class="flex space-x-1 rounded-lg bg-zinc-100 p-1">
                <Tab as="template" v-slot="{ selected }">
                  <button
                    :class="[
                      'w-full rounded-md py-3 px-4 text-sm font-medium leading-5',
                      'ring-white ring-opacity-60 ring-offset-2 ring-offset-blue-400',
                      'focus:outline-none focus:ring-2 transition-all',
                      selected
                        ? 'bg-white text-zinc-700 shadow'
                        : 'text-zinc-500 hover:bg-white/[0.12] hover:text-zinc-700',
                    ]"
                    @click="selectPedestrianType('none')"
                  >
                    {{ t("pedestrian.none") }}
                  </button>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <button
                    :class="[
                      'w-full rounded-md py-3 px-4 text-sm font-medium leading-5',
                      'ring-white ring-opacity-60 ring-offset-2 ring-offset-blue-400',
                      'focus:outline-none focus:ring-2 transition-all',
                      selected
                        ? 'bg-white text-zinc-700 shadow'
                        : 'text-zinc-500 hover:bg-white/[0.12] hover:text-zinc-700',
                    ]"
                    @click="selectPedestrianType('wheelchair')"
                  >
                    {{ t("pedestrian.wheelchair") }}
                  </button>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <button
                    :class="[
                      'w-full rounded-md py-3 px-4 text-sm font-medium leading-5',
                      'ring-white ring-opacity-60 ring-offset-2 ring-offset-blue-400',
                      'focus:outline-none focus:ring-2 transition-all',
                      selected
                        ? 'bg-white text-zinc-700 shadow'
                        : 'text-zinc-500 hover:bg-white/[0.12] hover:text-zinc-700',
                    ]"
                    @click="selectPedestrianType('blind')"
                  >
                    {{ t("pedestrian.blind") }}
                  </button>
                </Tab>
              </TabList>
            </TabGroup>
          </div>

          <!-- Bicycle Type Setting -->
          <div>
            <h3 class="text-lg font-semibold text-zinc-800 mb-2">{{ t("bicycleType") }}</h3>
            <p class="text-sm text-zinc-600 mb-4">{{ t("bicycleType.help") }}</p>
            <TabGroup
              :default-index="['hybrid', 'road', 'cross', 'mountain'].indexOf(preferences.bicycle_type || 'hybrid')"
            >
              <TabList class="grid grid-cols-2 gap-2 rounded-lg bg-zinc-100 p-2">
                <Tab as="template" v-slot="{ selected }">
                  <button
                    :class="[
                      'rounded-md px-3 py-3 text-sm font-medium leading-5',
                      'ring-white ring-opacity-60 ring-offset-2 ring-offset-blue-400',
                      'focus:outline-none focus:ring-2 transition-all',
                      selected
                        ? 'bg-white text-zinc-700 shadow'
                        : 'text-zinc-500 hover:bg-white/[0.12] hover:text-zinc-700',
                    ]"
                    @click="selectBicycleType('hybrid')"
                  >
                    {{ t("bicycle.hybrid") }}
                  </button>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <button
                    :class="[
                      'rounded-md px-3 py-3 text-sm font-medium leading-5',
                      'ring-white ring-opacity-60 ring-offset-2 ring-offset-blue-400',
                      'focus:outline-none focus:ring-2 transition-all',
                      selected
                        ? 'bg-white text-zinc-700 shadow'
                        : 'text-zinc-500 hover:bg-white/[0.12] hover:text-zinc-700',
                    ]"
                    @click="selectBicycleType('road')"
                  >
                    {{ t("bicycle.road") }}
                  </button>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <button
                    :class="[
                      'rounded-md px-3 py-3 text-sm font-medium leading-5',
                      'ring-white ring-opacity-60 ring-offset-2 ring-offset-blue-400',
                      'focus:outline-none focus:ring-2 transition-all',
                      selected
                        ? 'bg-white text-zinc-700 shadow'
                        : 'text-zinc-500 hover:bg-white/[0.12] hover:text-zinc-700',
                    ]"
                    @click="selectBicycleType('cross')"
                  >
                    {{ t("bicycle.cross") }}
                  </button>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <button
                    :class="[
                      'rounded-md px-3 py-3 text-sm font-medium leading-5',
                      'ring-white ring-opacity-60 ring-offset-2 ring-offset-blue-400',
                      'focus:outline-none focus:ring-2 transition-all',
                      selected
                        ? 'bg-white text-zinc-700 shadow'
                        : 'text-zinc-500 hover:bg-white/[0.12] hover:text-zinc-700',
                    ]"
                    @click="selectBicycleType('mountain')"
                  >
                    {{ t("bicycle.mountain") }}
                  </button>
                </Tab>
              </TabList>
            </TabGroup>
          </div>

          <!-- Motorcycle Type Setting -->
          <div>
            <h3 class="text-lg font-semibold text-zinc-800 mb-2">{{ t("ptwType") }}</h3>
            <p class="text-sm text-zinc-600 mb-4">{{ t("ptwType.help") }}</p>
            <TabGroup :default-index="['motorcycle', 'moped'].indexOf(preferences.ptw_type || 'motorcycle')">
              <TabList class="flex space-x-1 rounded-lg bg-zinc-100 p-1">
                <Tab as="template" v-slot="{ selected }">
                  <button
                    :class="[
                      'w-full rounded-md py-3 px-4 text-sm font-medium leading-5',
                      'ring-white ring-opacity-60 ring-offset-2 ring-offset-blue-400',
                      'focus:outline-none focus:ring-2 transition-all',
                      selected
                        ? 'bg-white text-zinc-700 shadow'
                        : 'text-zinc-500 hover:bg-white/[0.12] hover:text-zinc-700',
                    ]"
                    @click="selectPtwType('motorcycle')"
                  >
                    {{ t("ptw.motorcycle") }}
                  </button>
                </Tab>
                <Tab as="template" v-slot="{ selected }">
                  <button
                    :class="[
                      'w-full rounded-md py-3 px-4 text-sm font-medium leading-5',
                      'ring-white ring-opacity-60 ring-offset-2 ring-offset-blue-400',
                      'focus:outline-none focus:ring-2 transition-all',
                      selected
                        ? 'bg-white text-zinc-700 shadow'
                        : 'text-zinc-500 hover:bg-white/[0.12] hover:text-zinc-700',
                    ]"
                    @click="selectPtwType('moped')"
                  >
                    {{ t("ptw.moped") }}
                  </button>
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
  language: Sprache
  theme: Design
  theme.system: System
  theme.dark: Dunkel
  theme.light: Hell
  preferredTransportMode: Bevorzugtes Verkehrsmittel
  preferredTransportMode.help: Dies wird als Standard für die Navigation verwendet.
  transport.pedestrian: Zu Fuß
  transport.bicycle: Fahrrad
  transport.motorcycle: Motorrad
  transport.car: Auto
  transport.publicTransit: Öffentliche Verkehrsmittel
  pedestrianType: Fußgänger-Typ
  pedestrianType.help: Wählen Sie dies, falls Sie Barrierefreiheit benötigen wie Ansagen oder Aufzüge.
  pedestrian.none: Standard
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
  language: Language
  theme: Theme
  theme.system: System
  theme.dark: Dark
  theme.light: Light
  preferredTransportMode: Preferred Transport Mode
  preferredTransportMode.help: This will be used as the default for navigation.
  transport.pedestrian: Walking
  transport.bicycle: Bicycle
  transport.motorcycle: Motorcycle
  transport.car: Car
  transport.publicTransit: Public Transit
  pedestrianType: Pedestrian Type
  pedestrianType.help: Select this if you need accessibility features like narration or elevators.
  pedestrian.none: Standard
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
