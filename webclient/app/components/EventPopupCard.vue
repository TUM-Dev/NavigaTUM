<script setup lang="ts">
import { mdiOpenInNew } from "@mdi/js";

const props = defineProps<{
  readonly name: string;
  readonly description: string;
  readonly imagePath?: string;
  /** Pre-resolved image URL (e.g. a local `blob:`) for previewing a not-yet-uploaded image; overrides `imagePath`. */
  readonly imageSrcOverride?: string | null;
  readonly startsAt: string;
  readonly endsAt: string;
  readonly orgCode: string;
  readonly orgNameDe: string;
  readonly orgNameEn: string;
}>();

const runtimeConfig = useRuntimeConfig();
const { locale, t } = useI18n({ useScope: "local" });

const TZ = "Europe/Berlin";

function berlinDateKey(date: Date): string {
  return new Intl.DateTimeFormat("en-CA", {
    timeZone: TZ,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).format(date);
}

const timeRange = computed(() => {
  const isDe = locale.value === "de";
  const bcp47 = isDe ? "de-DE" : "en-GB";
  const start = new Date(props.startsAt);
  const end = new Date(props.endsAt);
  const time = new Intl.DateTimeFormat(bcp47, {
    timeZone: TZ,
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  });
  const date = new Intl.DateTimeFormat(bcp47, {
    timeZone: TZ,
    day: "numeric",
    month: isDe ? "long" : "short",
  });
  if (berlinDateKey(start) === berlinDateKey(end)) {
    return `${date.format(start)}, ${time.format(start)}-${time.format(end)}`;
  }
  return `${date.format(start)}, ${time.format(start)} - ${date.format(end)}, ${time.format(end)}`;
});

const badge = computed(() => {
  const start = Date.parse(props.startsAt);
  const end = Date.parse(props.endsAt);
  const now = Date.now();
  return now >= start && now <= end
    ? {
        label: t("badge_now"),
        class: "bg-green-100 text-green-800 dark:bg-green-900/50 dark:text-green-200",
      }
    : {
        label: t("badge_soon"),
        class: "bg-amber-100 text-amber-800 dark:bg-amber-900/50 dark:text-amber-200",
      };
});

const orgName = computed(() => (locale.value === "de" ? props.orgNameDe : props.orgNameEn));

const imageSrc = computed(
  () => props.imageSrcOverride ?? `${runtimeConfig.public.cdnURL}${props.imagePath ?? ""}`
);
</script>

<template>
  <article
    class="bg-white text-zinc-800 dark:bg-zinc-800 dark:text-zinc-100 w-72 max-w-full overflow-hidden rounded-sm shadow-lg"
  >
    <figure class="bg-zinc-200 dark:bg-zinc-700 aspect-video">
      <img
        :src="imageSrc"
        :alt="t('image_alt', [props.name])"
        loading="lazy"
        class="block h-full w-full object-cover"
      />
    </figure>
    <div class="p-2.5">
      <header class="mb-2 flex flex-wrap items-start justify-between gap-2">
        <h3 class="text-base font-semibold leading-tight">{{ props.name }}</h3>
        <span
          :class="[badge.class, 'inline-flex shrink-0 rounded-sm px-2 py-0.5 text-xs font-medium']"
        >
          {{ badge.label }}
        </span>
      </header>
      <p class="text-zinc-600 dark:text-zinc-300 mb-2 text-sm">
        <time :datetime="props.startsAt">{{ timeRange }}</time>
      </p>
      <p class="text-zinc-700 dark:text-zinc-200 mb-3 text-sm whitespace-pre-line">
        {{ props.description }}
      </p>
      <NuxtLinkLocale
        :to="`/view/${props.orgCode}`"
        class="text-blue-600 dark:text-blue-300 focusable inline-flex items-center gap-1 text-sm hover:underline"
        :aria-label="t('open_org', [orgName])"
      >
        {{ orgName }}
        <MdiIcon :path="mdiOpenInNew" :size="14" />
      </NuxtLinkLocale>
    </div>
  </article>
</template>

<i18n lang="yaml">
de:
  image_alt: "Bild zur Veranstaltung '{0}'"
  open_org: "Veranstalter '{0}' auf NavigaTUM öffnen"
  badge_now: "Gerade aktiv"
  badge_soon: "Beginnt bald"
en:
  image_alt: "Photo for the event '{0}'"
  open_org: "Open organiser '{0}' on NavigaTUM"
  badge_now: "Happening now"
  badge_soon: "Starting soon"
</i18n>
