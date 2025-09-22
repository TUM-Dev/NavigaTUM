<script setup lang="ts">
import { Tab, TabGroup, TabList } from "@headlessui/vue";
import {
  mdiAccountMultiple,
  mdiBike,
  mdiBikeFast,
  mdiBus,
  mdiCar,
  mdiEye,
  mdiImageFilterHdr,
  mdiMonitor,
  mdiMoonWaningCrescent,
  mdiMotorbike,
  mdiRoadVariant,
  mdiSpeedometer,
  mdiTune,
  mdiWalk,
  mdiWheelchairAccessibility,
  mdiWhiteBalanceSunny,
} from "@mdi/js";

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
</script>

<template>
  <div>
    <!-- Trigger Button -->
    <button
      id="preferences"
      class="focusable relative flex rounded-full bg-transparent p-2 text-sm ring-2 ring-white ring-opacity-0 hover:bg-zinc-100/10 hover:ring-opacity-20 focus:outline-none focus:ring-opacity-100"
      @click="isOpen = true"
    >
      <span class="absolute -inset-1.5" />
      <span class="sr-only">Open preferences menu</span>
      <MdiIcon :path="mdiTune" :size="28" class="text-zinc-900" />
    </button>

    <!-- Modal Dialog -->
    <ClientOnly>
      <LazyModal v-model="isOpen" :title="t('preferences')" class="bg-white" @close="isOpen = false">
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
                    @click="colorMode.value = 'system'"
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
                    @click="colorMode.value = 'light'"
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
                    @click="colorMode.value = 'dark'"
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
                    @click="locale = 'de'"
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
                    @click="locale = 'en'"
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
                    @click="updatePreference('route_costing', 'pedestrian')"
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
                    @click="updatePreference('route_costing', 'bicycle')"
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
                    @click="updatePreference('route_costing', 'motorcycle')"
                  >
                    <div class="flex items-center gap-2">
                      <MdiIcon :path="mdiMotorbike" :size="20" />
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
                    @click="updatePreference('route_costing', 'car')"
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
                    @click="updatePreference('route_costing', 'public_transit')"
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
            <TabGroup
              :default-index="
                preferences.pedestrian_type === 'blind' ? 2 : preferences.pedestrian_type === 'wheelchair' ? 1 : 0
              "
            >
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
                    @click="updatePreference('pedestrian_type', 'standard')"
                  >
                    <div class="flex items-center justify-center gap-2">
                      <MdiIcon :path="mdiAccountMultiple" :size="16" />
                      {{ t("pedestrian.standard") }}
                    </div>
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
                    @click="updatePreference('pedestrian_type', 'wheelchair')"
                  >
                    <div class="flex items-center justify-center gap-2">
                      <MdiIcon :path="mdiWheelchairAccessibility" :size="16" />
                      {{ t("pedestrian.wheelchair") }}
                    </div>
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
                    @click="updatePreference('pedestrian_type', 'blind')"
                  >
                    <div class="flex items-center justify-center gap-2">
                      <MdiIcon :path="mdiEye" :size="16" />
                      {{ t("pedestrian.blind") }}
                    </div>
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
              <TabList class="grid grid-cols-4 gap-2 rounded-lg bg-zinc-100 p-2">
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
                    @click="updatePreference('bicycle_type', 'hybrid')"
                  >
                    <div class="flex items-center justify-center gap-2">
                      <MdiIcon :path="mdiBike" :size="16" />
                      {{ t("bicycle.hybrid") }}
                    </div>
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
                    @click="updatePreference('bicycle_type', 'road')"
                  >
                    <div class="flex items-center justify-center gap-2">
                      <MdiIcon :path="mdiSpeedometer" :size="16" />
                      {{ t("bicycle.road") }}
                    </div>
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
                    @click="updatePreference('bicycle_type', 'cross')"
                  >
                    <div class="flex items-center justify-center gap-2">
                      <MdiIcon :path="mdiRoadVariant" :size="16" />
                      {{ t("bicycle.cross") }}
                    </div>
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
                    @click="updatePreference('bicycle_type', 'mountain')"
                  >
                    <div class="flex items-center justify-center gap-2">
                      <MdiIcon :path="mdiImageFilterHdr" :size="16" />
                      {{ t("bicycle.mountain") }}
                    </div>
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
                    @click="updatePreference('ptw_type', 'motorcycle')"
                  >
                    <div class="flex items-center justify-center gap-2">
                      <MdiIcon :path="mdiMotorbike" :size="16" />
                      {{ t("ptw.motorcycle") }}
                    </div>
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
                    @click="updatePreference('ptw_type', 'moped')"
                  >
                    <div class="flex items-center justify-center gap-2">
                      <MdiIcon :path="mdiBike" :size="16" />
                      {{ t("ptw.moped") }}
                    </div>
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
