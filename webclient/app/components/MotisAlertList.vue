<script setup lang="ts">
import type { components } from "~/api_types";

type AlertResponse = components["schemas"]["AlertResponse"];

interface Props {
  alerts: readonly AlertResponse[];
  title?: string;
  size?: "sm" | "md" | "lg";
  showDescription?: boolean;
  showCauseEffect?: boolean;
  maxVisible?: number;
}

const props = withDefaults(defineProps<Props>(), {
  size: "md",
  showDescription: true,
  showCauseEffect: true,
  maxVisible: 3,
});

const { t } = useI18n({ useScope: "local" });

const showAll = ref(false);

const visibleAlerts = computed(() => {
  if (showAll.value || props.alerts.length <= props.maxVisible) {
    return props.alerts;
  }
  return props.alerts.slice(0, props.maxVisible);
});

const hasMoreAlerts = computed(() => {
  return props.alerts.length > props.maxVisible;
});

const remainingCount = computed(() => {
  return props.alerts.length - props.maxVisible;
});

// Sort alerts by severity (severe first)
const sortedAlerts = computed(() => {
  return [...props.alerts].sort((a, b) => {
    const severityOrder = { severe: 0, warning: 1, info: 2, unknown: 3 };
    const aSeverity = severityOrder[a.severity_level as keyof typeof severityOrder] ?? 4;
    const bSeverity = severityOrder[b.severity_level as keyof typeof severityOrder] ?? 4;
    return aSeverity - bSeverity;
  });
});
</script>

<template>
  <div v-if="alerts.length > 0" class="space-y-2">
    <!-- Title -->
    <div v-if="title" class="text-xs text-zinc-600 font-medium mb-1">
      {{ title }}
    </div>

    <!-- Alerts -->
    <div class="space-y-2">
      <MotisAlert
        v-for="(alert, index) in visibleAlerts"
        :key="`alert-${index}`"
        :alert="alert"
        :size="size"
        :show-description="showDescription"
        :show-cause-effect="showCauseEffect"
      />
    </div>

    <!-- Show More/Less Toggle -->
    <div v-if="hasMoreAlerts" class="mt-2">
      <button
        @click="showAll = !showAll"
        class="text-xs text-zinc-600 hover:text-zinc-800 underline hover:no-underline transition-colors"
      >
        <span v-if="!showAll">
          {{ t("show_more_alerts", remainingCount) }}
        </span>
        <span v-else>
          {{ t("show_fewer_alerts") }}
        </span>
      </button>
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  show_more_alerts: "{count} weitere Meldung anzeigen | {count} weitere Meldungen anzeigen"
  show_fewer_alerts: Weniger anzeigen

en:
  show_more_alerts: "Show {count} more alert | Show {count} more alerts"
  show_fewer_alerts: Show fewer
</i18n>
