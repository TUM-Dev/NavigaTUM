<script setup lang="ts">
import { mdiClockOutline, mdiClose, mdiEmailOutline, mdiMapMarkerOutline } from "@mdi/js";
import hiringIllustration from "~/assets/hiring_navigatum.svg";

// Bump the suffix to re-show the popup after a substantially changed posting.
const NOTICE_ID = "hiring-werkstudent-2026";
const APPLY_MAILTO = "mailto:cloud@it.tum.de?subject=Bewerbung%20NavigaTUM";

const { t, tm, rt } = useI18n({ useScope: "local" });

// No `default` factory; see `userPreferences.ts` (avoids per-user `Set-Cookie` on `swr` responses).
const dismissedNotices = useCookie<string[] | null>("dismissedNotices");
const isOpen = ref(false);

onMounted(() => {
  if (!dismissedNotices.value?.includes(NOTICE_ID)) isOpen.value = true;
});

// Persist the dismissal so the popup stays closed on the next visit. Escape and the backdrop route
// through Modal's `close` event; the in-illustration button calls `close` directly.
function dismiss() {
  if (!dismissedNotices.value?.includes(NOTICE_ID))
    dismissedNotices.value = [...(dismissedNotices.value ?? []), NOTICE_ID];
}

function close() {
  dismiss();
  isOpen.value = false;
}

const facts = computed(() => [
  { icon: mdiClockOutline, label: t("facts.availability") },
  { icon: mdiMapMarkerOutline, label: t("facts.location") },
  { icon: mdiClockOutline, label: t("facts.hours") },
]);

const columns = ["responsibilities", "requirements", "benefits"] as const;
</script>

<template>
  <Modal v-model="isOpen" :title="t('title')" chromeless class="max-w-2xl" @close="dismiss">
    <div class="relative flex max-h-[90vh] w-full max-w-2xl flex-col overflow-hidden rounded-md bg-white shadow-2xl dark:bg-zinc-800">
      <button
        type="button"
        :aria-label="t('close')"
        class="focusable absolute end-3 top-3 z-10 rounded-full bg-white/80 p-2 text-zinc-700 shadow-sm hover:bg-white hover:text-blue-700 dark:bg-zinc-900/70 dark:text-zinc-200 dark:hover:bg-zinc-900"
        @click="close"
      >
        <MdiIcon :path="mdiClose" :size="18" />
      </button>

      <div class="overflow-y-auto">
        <!-- Fixed light panel so the illustration's transparent corners read the same in both themes. -->
        <div class="bg-[#f3f9ff]">
          <img :src="hiringIllustration" :alt="t('illustration_alt')" class="mx-auto block h-auto w-full max-w-md" />
        </div>

        <div class="flex flex-col gap-5 p-6">
          <div class="flex flex-col gap-3">
            <h2 class="text-zinc-800 text-xl font-bold dark:text-zinc-100">{{ t("title") }}</h2>
            <ul class="flex flex-wrap gap-2">
              <li
                v-for="fact in facts"
                :key="fact.label"
                class="text-blue-900 dark:text-blue-50 bg-blue-100 dark:bg-blue-800 flex items-center gap-1.5 rounded-full px-3 py-1 text-sm font-medium"
              >
                <MdiIcon :path="fact.icon" :size="14" />
                {{ fact.label }}
              </li>
            </ul>
          </div>

          <p class="text-zinc-600 dark:text-zinc-300 text-pretty text-sm leading-6">
            <EmphasizedText :text="t('intro')" />
          </p>

          <div class="grid gap-5 sm:grid-cols-3">
            <section v-for="column in columns" :key="column" class="flex flex-col gap-2">
              <h3 class="text-zinc-800 dark:text-zinc-100 text-sm font-semibold">{{ t(`${column}.heading`) }}</h3>
              <ul class="flex flex-col gap-1.5">
                <li
                  v-for="(item, index) in tm(`${column}.items`)"
                  :key="index"
                  class="text-zinc-600 dark:text-zinc-300 ps-4 text-sm leading-5 -indent-4 before:pe-1.5 before:text-blue-500 before:content-['▸']"
                >
                  <EmphasizedText :text="rt(item)" />
                </li>
              </ul>
            </section>
          </div>

          <div class="bg-blue-50 dark:bg-blue-900/40 flex flex-col gap-1 rounded-md p-4">
            <h3 class="text-zinc-800 dark:text-zinc-100 text-sm font-semibold">{{ t("about.heading") }}</h3>
            <p class="text-zinc-600 dark:text-zinc-300 text-pretty text-sm leading-6">
              <EmphasizedText :text="t('about.body')" />
            </p>
          </div>

          <div class="flex flex-col gap-3 border-t border-zinc-200 pt-4 dark:border-zinc-700">
            <p class="text-zinc-600 dark:text-zinc-300 text-pretty text-sm leading-6">
              <EmphasizedText :text="t('call_to_action')" />
            </p>
            <div class="flex flex-wrap items-center gap-x-4 gap-y-2">
              <a
                :href="APPLY_MAILTO"
                class="focusable text-md text-blue-50 dark:text-blue-900 bg-blue-500 dark:bg-blue-400 hover:bg-blue-600 dark:hover:bg-blue-300 hover:text-white dark:hover:text-black flex flex-row items-center gap-1.5 rounded-sm px-4 py-1.5"
              >
                <MdiIcon :path="mdiEmailOutline" :size="16" />
                {{ t("apply") }}
              </a>
              <a
                :href="APPLY_MAILTO"
                class="focusable text-blue-600 dark:text-blue-300 text-sm hover:underline"
              >
                cloud@it.tum.de
              </a>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Modal>
</template>

<i18n lang="yaml">
de:
  close: Schließen
  illustration_alt: Illustration des TUM-Hauptgebäudes mit einer Person, die sich mit dem Smartphone orientiert
  title: Studentische Hilfskraft (m/w/d) bei NavigaTUM
  facts:
    availability: Ab sofort
    location: Innenstadt München
    hours: 6–15 Stunden/Woche
  intro: Schon mal verzweifelt einen Raum gesucht? Oder nicht gewusst, wo genau deine Prüfung stattfindet? Genau solche Probleme lösen wir bei NavigaTUM – mit **präziser Indoor-Navigation** auf Basis von Gebäudedaten. Du möchtest verstehen, wie Gebäude digital abgebildet und navigierbar gemacht werden, und selbst aktiv mitarbeiten? Dann bist du bei uns genau richtig. Wir suchen ab sofort **mehrere studentische Hilfskräfte (m/w/d)** für das Projekt NavigaTUM.
  responsibilities:
    heading: Was dich erwartet
    items:
      - Aufbereitung von IFC-Gebäudedaten für die Weiterverarbeitung
      - Kartierung in JOSM (Java OpenStreetMap Editor) pflegen und überführen
      - Gebäude- und Kartendaten zusammenführen und strukturieren
      - Datenfehler identifizieren und beheben
      - Qualitätssicherung und kontinuierliche Verbesserung der Datensätze
  requirements:
    heading: Was du mitbringen solltest
    items:
      - Immatrikulation, bevorzugt in Bauingenieurwesen, Architektur, Geodäsie oder Informatik
      - Interesse an Gebäudemodellen, Karten und digitalen Datenstrukturen
      - Sorgfältige, strukturierte Arbeitsweise
      - Grundverständnis technischer Zusammenhänge
      - Bereitschaft, dich in neue Tools (z. B. JOSM, IFC) einzuarbeiten
  benefits:
    heading: Was wir dir bieten
    items:
      - Praxiserfahrung an einem realen Navigationssystem mit gesellschaftlichem Nutzen
      - Einblicke in BIM, Geodaten und digitale Infrastruktur
      - Flexible Arbeitszeiten, ideal mit dem Studium vereinbar
      - Ein motiviertes, interdisziplinäres Team
      - Arbeitsplatz zentral in München oder teilweise remote
      - Längerfristige Mitarbeit möglich
  about:
    heading: Über uns
    body: NavigaTUM ist eine Plattform zur Indoor- und Campus-Navigation. Ziel ist es, Gebäude (z. B. Hochschulen) digital zugänglich zu machen und die Navigation innerhalb von Gebäuden zu ermöglichen. Grundlage sind unter anderem BIM-Daten (IFC), die in kartierbare Strukturen überführt und in OpenStreetMap (OSM) integriert werden.
  call_to_action: Neugierig geworden? Dann freuen wir uns über deine Bewerbung mit kurzem Motivationsschreiben und Lebenslauf – mit dem Betreff **„Bewerbung NavigaTUM“**.
  apply: Jetzt bewerben
en:
  close: Close
  illustration_alt: Illustration of the TUM main building with a person finding their way using a smartphone
  title: Student assistant (m/f/d) at NavigaTUM
  facts:
    availability: Starting now
    location: Munich city centre
    hours: 6–15 hours/week
  intro: Ever searched in vain for a room? Or not known where exactly your exam takes place? These are precisely the problems we solve at NavigaTUM – with **accurate indoor navigation** based on building data. Want to understand how buildings are mapped digitally and made navigable, and help build it yourself? Then you are in the right place. We are looking for **several student assistants (m/f/d)** for the NavigaTUM project, starting now.
  responsibilities:
    heading: What you will do
    items:
      - Prepare IFC building data for further processing
      - Maintain and transfer mapping in JOSM (Java OpenStreetMap editor)
      - Merge and structure building and map data
      - Identify and fix data errors
      - Quality assurance and continuous improvement of the datasets
  requirements:
    heading: What you bring
    items:
      - Enrolment, preferably in civil engineering, architecture, geodesy, or computer science
      - Interest in building models, maps, and digital data structures
      - A careful, structured way of working
      - A basic understanding of technical concepts
      - Willingness to learn new tools (e.g. JOSM, IFC)
  benefits:
    heading: What we offer
    items:
      - Hands-on experience on a real navigation system with social value
      - Insights into BIM, geodata, and digital infrastructure
      - Flexible hours, easy to combine with your studies
      - A motivated, interdisciplinary team
      - A workplace central in Munich, or partly remote
      - Longer-term collaboration possible
  about:
    heading: About us
    body: NavigaTUM is a platform for indoor and campus navigation. Its goal is to make buildings (e.g. universities) digitally accessible and to enable navigation inside them. It builds on BIM data (IFC), among other sources, which is converted into mappable structures and integrated into OpenStreetMap (OSM).
  call_to_action: Curious? We look forward to your application with a short cover letter and CV – with the subject **“Bewerbung NavigaTUM”**.
  apply: Apply now
</i18n>
