<script setup lang="ts">
import { getLocalStorageWithExpiry } from "@/composables/storage";
import { useDetailsStore } from "@/stores/details";
import { useGlobalStore } from "@/stores/global";
import { useI18n } from "vue-i18n";
import type { Coord } from "@/stores/global";

const { t } = useI18n({ inheritLocale: true, useScope: "global" });
const state = useDetailsStore();
type CurrentEdits = { [index: string]: Coord };
function _getFeedbackSubject(currentEdits: CurrentEdits) {
  if (Object.keys(currentEdits).length > 1) {
    return `[${state.data?.id} et.al.]: ` + t("feedback.coordinatepicker.edit_coordinates_subject");
  }

  const subjectPrefix = `[${state.data?.id}]: `;
  const subjectMsg =
    Object.keys(currentEdits).length === 0 ? "" : t("feedback.coordinatepicker.edit_coordinate_subject");

  // The subject backup is only loaded (and supported) when a single
  // entry is being edited
  if (
    state.coord_picker.subject_backup &&
    state.coord_picker.backup_id === state.data?.id &&
    state.coord_picker.subject_backup !== subjectPrefix
  ) {
    const backup = state.coord_picker.subject_backup;
    state.coord_picker.subject_backup = null;
    return backup;
  }
  return subjectPrefix + subjectMsg;
}
function _getFeedbackBody(currentEdits: CurrentEdits) {
  if (!state.data) {
    return "Data has not been loaded yet but Feedback is open. This is a bug. Please describe how you got here.";
  }
  // Look up whether there is a backup of the body and extract the section
  // that is not the coordinate
  let actionMsg = "";
  if (state.coord_picker.body_backup && state.coord_picker.backup_id === state.data.id) {
    const parts = state.coord_picker.body_backup.split("\n```");
    if (parts.length === 1) {
      actionMsg = parts[0];
    } else {
      actionMsg = parts[0] + parts[1].split("```").slice(1).join("\n");
    }

    state.coord_picker.body_backup = null;
  }

  if (Object.keys(currentEdits).length === 0) {
    // For no edits, don't show a badly formatted message
    // (This is "" if there was no backup)
    return actionMsg;
  }

  const defaultActionMsg =
    state.data.coords.accuracy === "building"
      ? t("feedback.coordinatepicker.add_coordinate")
      : t("feedback.coordinatepicker.correct_coordinate");
  actionMsg = actionMsg || defaultActionMsg;

  if (Object.keys(currentEdits).length > 1) {
    // The body backup is discarded if more than a single entry
    // is being edited (because then it is not supported).
    actionMsg = t("feedback.coordinatepicker.edit_multiple_coordinates");
  }

  let editStr = "";
  Object.entries(currentEdits).forEach(([key, value]) => {
    editStr += `"${key}": { lat: ${value.coords.lat}, lon: ${value.coords.lon} }\n`;
  });

  return `${actionMsg}\n\`\`\`yaml\n${editStr}\`\`\``;
}

defineExpose({
  openFeedbackForm,
});
function openFeedbackForm(addLocationPicker: EventListener | null = null) {
  // The feedback form is opened. This may be prefilled with previously corrected coordinates.
  // Maybe get the old coordinates from localstorage
  const currentEdits = getLocalStorageWithExpiry<CurrentEdits>("feedback-coords", {});
  const body = _getFeedbackBody(currentEdits);
  const subject = _getFeedbackSubject(currentEdits);

  if (addLocationPicker !== null) {
    window.setTimeout(
      () => document.getElementById("feedback-coordinate-picker")?.addEventListener("click", addLocationPicker),
      100
    );
  }

  useGlobalStore().openFeedback("entry", subject, body);
}
</script>

<template>
  <button
    class="btn btn-link btn-action btn-sm"
    :title="$t('view_view.header.feedback')"
    @click="openFeedbackForm()"
    data-cy="open-feedback-details"
  >
    <i class="icon icon-flag" />
  </button>
</template>
