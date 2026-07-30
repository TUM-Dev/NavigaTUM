<script setup lang="ts">
import { mdiClockOutline, mdiClose, mdiEmailOutline, mdiMapMarkerOutline } from "@mdi/js";
import hiringIllustration from "~/assets/hiring_navigatum.svg";

// Bump the suffix to re-show the popup after a substantially changed posting.
const NOTICE_ID = "hiring-werkstudent-2026";
const APPLY_MAILTO = "mailto:cloud@it.tum.de?subject=Bewerbung%20Produkt%20Support";

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
  title: Studentische Hilfskraft (m/w/d) für den Produktsupport
  facts:
    availability: Ab sofort
    location: Innenstadt München
    hours: 6-12 Stunden/Woche
  intro: Du willst mehr als nur Tickets abarbeiten? Du möchtest verstehen, wie ein Produkt entsteht – und es aktiv mitgestalten? Dann bist du bei uns genau richtig! Wir suchen mehrere engagierte und technikaffine Studierende, die nicht nur unseren Produktsupport unterstützen, sondern ihn gemeinsam mit uns weiterentwickeln.
  responsibilities:
    heading: Was dich erwartet
    items:
      - Du bist die erste Anlaufstelle für Nutzeranfragen rund um unsere Softwarelösungen
      - Du arbeitest im direkten Austausch mit der Entwicklung; ohne Umwege, mit Zugang zu relevanten Informationen aus erster Hand
      - Du bringst dich aktiv ein, verbesserst Prozesse im Support und hilfst mit, unsere Tools und Abläufe effizienter und nutzerfreundlicher zu gestalten
      - Du dokumentierst und strukturierst komplexe technische Inhalte – verständlich und klar
      - Du arbeitest agil im Team und hast Raum, Verantwortung zu übernehmen und neue Ideen umzusetzen
  requirements:
    heading: Was du mitbringen solltest
    items:
      - Du studierst in einem MINT-Studiengang mit starker IT-Affinität
      - Du hast ein gutes Gespür für Technik und keine Scheu vor komplexen Systemen
      - Du arbeitest strukturiert, eigenverantwortlich und serviceorientiert
      - Du hast erste Erfahrungen mit agilen Arbeitsmethoden (z.B. Scrum, Kanban) oder Lust, dich damit vertraut zu machen
      - Du kommunizierst sicher auf Deutsch (mind. C1) und Englisch
  benefits:
    heading: Was wir bieten
    items:
      - Praxiserfahrung mit echter Verantwortung
      - kein Kaffeekochen, sondern mitgestalten
      - Einblicke in agile Produktentwicklung, aktuelle Technologien und digitale Transformationsprozesse
      - Flexible Arbeitszeiten, die sich mit deinem Studium vereinbaren lassen
      - Ein Arbeitsplatz mitten in München mit kollegialem Teamspirit und moderner Infrastruktur
      - Die Chance, langfristig Teil eines zukunftsorientierten Umfelds an der TUM zu sein
  about:
    heading: Neugierig geworden?
  call_to_action: Dann freuen wir uns als Cloud & IAM Team des IT Managements auf Deine Bewerbung mit kurzem Motivationsschreiben und Lebenslauf. Bitte sende deine Unterlagen mit dem Betreff Bewerbung Produkt Support an cloud@it.tum.de. Wir freuen uns darauf, Dich kennenzulernen!
  apply: Jetzt bewerben
en:
  close: Close
  illustration_alt: Illustration of the TUM main building with a person finding their way using a smartphone
  title: Student assistant (m/f/d) for product support
  facts:
    availability: Starting now
    location: Munich city centre
    hours: 6-12 hours/week
  intro: Do you want to do more than just churn through tickets? Would you like to understand how a product is created—and actively help shape it? Then you've come to the right place! We are looking for several dedicated, tech-savvy students who want to not only support our product support team but also help us develop it further.
  responsibilities:
    heading: What to expect
    items:
      - You are the first point of contact for user inquiries regarding our software solutions
      - You work in direct collaboration with the development team—no detours, with access to relevant information firsthand
      - You actively contribute by improving support processes and helping to make our tools and workflows more efficient and user-friendly
      - You document and structure complex technical content clearly and understandably
      - You work in an agile team environment with the opportunity to take on responsibility and implement new ideas
  requirements:
    heading: What you bring
    items:
      - You are currently pursuing a STEM degree with a strong interest in IT
      - You have a knack for technology and aren't intimidated by complex systems
      - You work in a structured, independent, and service-oriented manner
      - You have initial experience with agile working methods (e.g., Scrum, Kanban) or an interest in getting familiar with them
      - You communicate confidently in German (min. C1 level) and English
  benefits:
    heading: What we offer
    items:
      - Practical experience with real responsibility
      - No making coffee—instead, you help shape our work
      - Insights into agile product development, current technologies, and digital transformation processes
      - Flexible working hours that fit around your studies
      - A workplace in the heart of Munich featuring a collaborative team spirit and modern infrastructure
      - The chance to become a long-term part of a forward-looking environment at TUM
  about:
    heading: Interested?
  call_to_action: We look forward to your application with a short cover letter and CV - with the subject **"Bewerbung Produkt Support"**.
  apply: Apply now
</i18n>