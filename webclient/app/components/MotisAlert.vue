<script setup lang="ts">
import { mdiAlertOctagon, mdiAlert, mdiInformationOutline, mdiBullhornOutline } from "@mdi/js";
import type { components } from "~/api_types";

type AlertResponse = components["schemas"]["AlertResponse"];

interface Props {
  alert: AlertResponse;
  size?: "sm" | "md" | "lg";
  showDescription?: boolean;
  showCauseEffect?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  size: "md",
  showDescription: true,
  showCauseEffect: true,
});

const { t } = useI18n({ useScope: "local" });

// Helper function to get alert severity class
const getAlertSeverityClass = (severity?: string | null) => {
  const baseClasses = "rounded-lg border";

  switch (severity) {
    case "severe":
      return `${baseClasses} bg-red-50 text-red-900 border-red-200`;
    case "warning":
      return `${baseClasses} bg-orange-50 text-orange-900 border-orange-200`;
    case "info":
      return `${baseClasses} bg-blue-50 text-blue-900 border-blue-200`;
    default:
      return `${baseClasses} bg-gray-50 text-gray-900 border-gray-200`;
  }
};

// Helper function to get icon color class
const getIconColorClass = (severity?: string | null) => {
  switch (severity) {
    case "severe":
      return "text-red-700";
    case "warning":
      return "text-orange-700";
    case "info":
      return "text-blue-700";
    default:
      return "text-gray-700";
  }
};

// Helper function to get alert icon
const getAlertIcon = (severity?: string | null) => {
  switch (severity) {
    case "severe":
      return mdiAlertOctagon;
    case "warning":
      return mdiAlert;
    case "info":
      return mdiInformationOutline;
    default:
      return mdiBullhornOutline;
  }
};

// Helper function to format alert cause
const formatAlertCause = (cause?: string | null) => {
  if (!cause) return null;
  const causeMap: { [key: string]: string } = {
    technical_problem: t("technical_problem"),
    strike: t("strike"),
    demonstration: t("demonstration"),
    accident: t("accident"),
    holiday: t("holiday"),
    weather: t("weather"),
    maintenance: t("maintenance"),
    construction: t("construction"),
    police_activity: t("police_activity"),
    medical_emergency: t("medical_emergency"),
    other_cause: t("other_cause"),
    unknown_cause: t("unknown_cause"),
  };
  return causeMap[cause] || cause;
};

// Helper function to format alert effect
const formatAlertEffect = (effect?: string | null) => {
  if (!effect) return null;
  const effectMap: { [key: string]: string } = {
    no_service: t("no_service"),
    reduced_service: t("reduced_service"),
    significant_delays: t("significant_delays"),
    detour: t("detour"),
    additional_service: t("additional_service"),
    modified_service: t("modified_service"),
    stop_moved: t("stop_moved"),
    no_effect: t("no_effect"),
    accessibility_issue: t("accessibility_issue"),
    other_effect: t("other_effect"),
    unknown_effect: t("unknown_effect"),
  };
  return effectMap[effect] || effect;
};

// Size-based classes
const sizeClasses = computed(() => {
  switch (props.size) {
    case "sm":
      return {
        container: "p-2",
        iconSize: 16,
        header: "text-xs font-medium",
        description: "text-xs",
        meta: "text-xs",
      };
    case "lg":
      return {
        container: "p-4",
        iconSize: 24,
        header: "text-base font-semibold",
        description: "text-sm",
        meta: "text-sm",
      };
    default: // md
      return {
        container: "p-3",
        iconSize: 20,
        header: "text-sm font-medium",
        description: "text-sm",
        meta: "text-xs",
      };
  }
});
</script>

<template>
  <div :class="[getAlertSeverityClass(alert.severity_level), sizeClasses.container]">
    <div class="flex items-start gap-2">
      <div class="flex-shrink-0 mt-0.5">
        <MdiIcon
          :path="getAlertIcon(alert.severity_level)"
          :size="sizeClasses.iconSize"
          :class="getIconColorClass(alert.severity_level)"
        />
      </div>

      <div class="flex-grow min-w-0">
        <!-- Header -->
        <div :class="[sizeClasses.header, 'mb-1']">
          {{ alert.header_text }}
        </div>

        <!-- Cause and Effect -->
        <div
          v-if="showCauseEffect && (alert.cause || alert.effect)"
          :class="[sizeClasses.meta, 'flex flex-wrap gap-3 mb-2 opacity-80']"
        >
          <span v-if="alert.cause" class="flex items-center gap-1">
            <span class="font-medium">{{ t("cause") }}:</span>
            {{ formatAlertCause(alert.cause) || alert.cause_detail }}
          </span>
          <span v-if="alert.effect" class="flex items-center gap-1">
            <span class="font-medium">{{ t("effect") }}:</span>
            {{ formatAlertEffect(alert.effect) || alert.effect_detail }}
          </span>
        </div>

        <!-- Description -->
        <div
          v-if="showDescription && alert.description_text"
          :class="[sizeClasses.description, 'opacity-90 leading-relaxed mb-2']"
        >
          {{ alert.description_text }}
        </div>

        <!-- Image (if available) -->
        <div v-if="alert.image_url" class="mb-2">
          <img
            :src="alert.image_url"
            :alt="alert.image_alternative_text || t('alert_image')"
            class="max-w-full h-auto rounded"
            loading="lazy"
          />
        </div>

        <!-- Additional info link -->
        <div v-if="alert.url" class="mt-2">
          <a
            :href="alert.url"
            target="_blank"
            rel="noopener noreferrer"
            :class="[sizeClasses.meta, 'inline-flex items-center gap-1 underline hover:no-underline']"
          >
            {{ t("more_info") }}
            <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
              />
            </svg>
          </a>
        </div>
      </div>
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  cause: Ursache
  effect: Auswirkung
  more_info: Mehr Infos
  alert_image: Meldungsbild
  technical_problem: Technisches Problem
  strike: Streik
  demonstration: Demonstration
  accident: Unfall
  holiday: Feiertag
  weather: Wetter
  maintenance: Wartung
  construction: Bauarbeiten
  police_activity: Polizeieinsatz
  medical_emergency: Medizinischer Notfall
  other_cause: Andere Ursache
  unknown_cause: Unbekannte Ursache
  no_service: Kein Service
  reduced_service: Eingeschränkter Service
  significant_delays: Erhebliche Verspätungen
  detour: Umleitung
  additional_service: Zusätzlicher Service
  modified_service: Geänderter Service
  stop_moved: Haltestelle verlegt
  no_effect: Keine Auswirkung
  accessibility_issue: Barrierefreiheitsproblem
  other_effect: Andere Auswirkung
  unknown_effect: Unbekannte Auswirkung

en:
  cause: Cause
  effect: Effect
  more_info: More info
  alert_image: Alert image
  technical_problem: Technical problem
  strike: Strike
  demonstration: Demonstration
  accident: Accident
  holiday: Holiday
  weather: Weather
  maintenance: Maintenance
  construction: Construction
  police_activity: Police activity
  medical_emergency: Medical emergency
  other_cause: Other cause
  unknown_cause: Unknown cause
  no_service: No service
  reduced_service: Reduced service
  significant_delays: Significant delays
  detour: Detour
  additional_service: Additional service
  modified_service: Modified service
  stop_moved: Stop moved
  no_effect: No effect
  accessibility_issue: Accessibility issue
  other_effect: Other effect
  unknown_effect: Unknown effect
</i18n>
