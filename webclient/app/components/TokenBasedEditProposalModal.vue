<script setup lang="ts">
import type { components } from "~/api_types";
import { useEditProposal } from "~/composables/editProposal";
import { useFeedbackSubmission } from "~/composables/feedbackSubmission";

type EditRequest = components["schemas"]["EditRequest"];

const open = defineModel<boolean>("open", { required: true });

const props = withDefaults(
  defineProps<{
    data: Pick<EditRequest, "additional_context"> & {
      edits: NonNullable<EditRequest["edits"]>;
      additions: NonNullable<EditRequest["additions"]>;
    };
    title?: string;
  }>(),
  { title: "" }
);

const emit = defineEmits<{
  beforeSubmit: [];
}>();

const { t } = useI18n({ useScope: "local" });
const submission = useFeedbackSubmission();
const privacyChecked = ref(false);
const editProposal = useEditProposal();
const modalTitle = computed(() => props.title || t("title"));

function closeForm() {
  open.value = false;
  editProposal.value.imageUpload.open = false;
  editProposal.value.locationPicker.open = false;
  submission.reset();
  privacyChecked.value = false;
}

function resetFormData() {
  // Reset form data after successful submission
  editProposal.value.data.additional_context = "";
  editProposal.value.data.edits = {};
  editProposal.value.data.additions = {};
  editProposal.value.selected.id = null;
  editProposal.value.selected.name = null;
  editProposal.value.imageUpload.selectedFile = null;
  editProposal.value.imageUpload.metadata = {
    author: "",
    license: { text: "", url: "" },
  };
  editProposal.value.locationPicker.lat = 0;
  editProposal.value.locationPicker.lon = 0;
}

async function sendForm() {
  emit("beforeSubmit");

  const hasContext = editProposal.value.data.additional_context.length >= 10;
  const hasEdits = Object.keys(editProposal.value.data.edits).length > 0;
  const hasAdditions = Object.keys(editProposal.value.data.additions).length > 0;
  if (!hasContext && !hasEdits && !hasAdditions) {
    submission.submitError.value = t("error.form.no_content");
    return;
  }

  const ok = await submission.submit(
    {
      additional_context: props.data.additional_context,
      edits: props.data.edits,
      additions: props.data.additions,
    },
    privacyChecked.value
  );
  if (ok) {
    resetFormData();
  } else {
    document.getElementById("token-modal-error")?.scrollIntoView({ behavior: "smooth" });
  }
}
</script>

<template>
  <Modal v-if="!submission.successUrl.value" v-model="open" :title="modalTitle" @close="closeForm">
    <Toast
      v-if="submission.submitError.value"
      id="token-modal-error"
      class="mb-4"
      :msg="submission.submitError.value"
      level="error"
    />
    <FeedbackValidationFailures class="mb-4" :failures="submission.validationFailures.value" />

    <div class="flex flex-col gap-1">
      <slot name="modal" />
      <div class="mt-6">
        <FeedbackConsentCheckbox id="privacy-checked" v-model="privacyChecked" />
      </div>
    </div>

    <div class="float-right flex flex-row-reverse gap-2">
      <FeedbackSubmitButton
        :submitting="submission.submitting.value"
        :blocked="submission.blockedByToken.value"
        @click="sendForm"
      />
      <Btn variant="linkButton" size="md" @click="closeForm">
        {{ t("cancel") }}
      </Btn>
    </div>
  </Modal>
  <Modal v-if="submission.successUrl.value" v-model="open" :title="t('thank_you')" @close="closeForm">
    <slot name="success" :success-url="submission.successUrl.value" />

    <Btn size="md" variant="primary" @click="closeForm">OK</Btn>
  </Modal>
</template>

<i18n lang="yaml">
de:
  title: Änderungen vorschlagen
  cancel: Abbrechen
  thank_you: Vielen Dank!
  error:
    form:
      no_content: "Fehler: Bitte füge zusätzlichen Kontext (mindestens 10 Zeichen), konkrete Änderungen oder neue Einträge hinzu"
en:
  title: Propose Changes
  cancel: Cancel
  thank_you: Thank you!
  error:
    form:
      no_content: "Error: Please add additional context (at least 10 characters), concrete edits, or new entries"
</i18n>
