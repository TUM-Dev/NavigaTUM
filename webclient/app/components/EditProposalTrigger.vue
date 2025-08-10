<script setup lang="ts">
import { useEditProposal } from "~/composables/editProposal";

interface Props {
  /**
   * The ID or name of the entity being edited
   */
  entityId?: string;
  /**
   * Pre-populate the form with coordinates
   */
  coordinates?: {
    lat: number;
    lon: number;
  };
  /**
   * Pre-populate additional context
   */
  context?: string;
  /**
   * Button variant style
   */
  variant?: "primary" | "secondary" | "link" | "linkButton";
  /**
   * Button size
   */
  size?: "sm" | "md" | "lg";
  /**
   * Custom button text (uses i18n default if not provided)
   */
  buttonText?: string;
}

const props = withDefaults(defineProps<Props>(), {
  variant: "secondary",
  size: "sm",
});

const { t } = useI18n({ useScope: "local" });
const editProposal = useEditProposal();

function openEditProposal() {
  // Pre-populate with provided data
  if (props.context) {
    editProposal.value.data.additional_context = props.context;
  } else if (props.entityId && editProposal.value.data.additional_context === "") {
    editProposal.value.data.additional_context = t("default_context", { entityId: props.entityId });
  }

  // Pre-populate coordinates if provided and we have an entityId
  if (props.coordinates && props.entityId) {
    // Initialize edit object if it doesn't exist
    if (!editProposal.value.data.edits[props.entityId]) {
      editProposal.value.data.edits[props.entityId] = {
        coordinate: null,
        image: null,
      };
    }
    editProposal.value.data.edits[props.entityId].coordinate = {
      lat: props.coordinates.lat,
      lon: props.coordinates.lon,
    };
  }

  editProposal.value.open = true;
}
</script>

<template>
  <Btn :variant="variant" :size="size" @click="openEditProposal" :aria-label="t('aria_label')">
    {{ buttonText || t("suggest_edit") }}
  </Btn>
</template>

<i18n lang="yaml">
de:
  suggest_edit: Änderung vorschlagen
  default_context: Verbesserungsvorschlag für {entityId}
  aria_label: Änderungen für diesen Standort vorschlagen
en:
  suggest_edit: Suggest Edit
  default_context: Improvement suggestion for {entityId}
  aria_label: Suggest changes for this location
</i18n>
