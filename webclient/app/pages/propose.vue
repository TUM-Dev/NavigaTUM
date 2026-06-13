<script setup lang="ts">
import type { LocationQueryRaw } from "vue-router";
import AddProposalForm from "~/components/AddProposalForm.vue";
import {
  type AdditionDraft,
  type AdditionKind,
  additionRegistry,
  emptyAdditionDraft,
} from "~/composables/additionSchema";
import { firstOrDefault } from "~/composables/common";
import { useFeedbackSubmission } from "~/composables/feedbackSubmission";
import { submissionBlock } from "~/composables/submissionGate";
import { useKnownOrgs } from "~/composables/useKnownOrgs";

const { t } = useI18n({ useScope: "local" });
const localePath = useLocalePath();
const route = useRoute();
const router = useRouter();
const submission = useFeedbackSubmission();

useSeoMeta({
  title: () => t("title"),
  description: () => t("description"),
});

function parseKind(value: string): AdditionKind | null {
  return value in additionRegistry ? (value as AdditionKind) : null;
}

function parsePositiveInt(value: string): number | null {
  const n = Number.parseInt(value, 10);
  return Number.isFinite(n) && n > 0 ? n : null;
}

function draftFromQuery(): AdditionDraft {
  const kind = parseKind(firstOrDefault(route.query.kind, ""));
  if (!kind) return emptyAdditionDraft();
  if (kind === "event") {
    const draft = additionRegistry.event.empty();
    const orgId = parsePositiveInt(firstOrDefault(route.query.org, ""));
    if (orgId !== null) draft.organising_org_id = orgId;
    return draft;
  }
  return additionRegistry[kind].empty();
}

const initialDraft = draftFromQuery();

// Wait so the Combobox can resolve the pre-set selection on first render.
if (initialDraft.kind === "event" && initialDraft.organising_org_id !== null) {
  await useKnownOrgs().ready();
}

// router.replace inside a watch during unmount races with the outgoing navigation.
let pageActive = false;
onMounted(() => {
  pageActive = true;
});
onBeforeUnmount(() => {
  pageActive = false;
});

const editProposal = useEditProposal();
watch(
  [
    () => editProposal.value.pendingAddition.kind,
    () => {
      const a = editProposal.value.pendingAddition;
      return a.kind === "event" ? a.organising_org_id : null;
    },
  ],
  ([kind, orgId]) => {
    if (!pageActive) return;
    const next: LocationQueryRaw = { ...route.query };
    if (kind) next.kind = kind;
    else delete next.kind;
    if (orgId === null) delete next.org;
    else next.org = String(orgId);
    const currentKind = firstOrDefault(route.query.kind, "");
    const currentOrg = firstOrDefault(route.query.org, "");
    if ((next.kind ?? "") === currentKind && (next.org ?? "") === currentOrg) return;
    router.replace({ query: next });
  }
);

const formRef = ref<InstanceType<typeof AddProposalForm> | null>(null);

// Locked "updating existing event" mode swaps the submit copy from proposing to updating.
const updatingEvent = computed(() => {
  const a = editProposal.value.pendingAddition;
  return a.kind === "event" && a.based_on !== null;
});

const privacyChecked = ref(false);

const sendBlock = computed(() =>
  submissionBlock({
    submitting: submission.submitting.value,
    succeeded: Boolean(submission.successUrl.value),
    blockedByToken: submission.blockedByToken.value,
    draftReady: formRef.value?.draftIsReady === true,
    privacyChecked: privacyChecked.value,
  })
);
const canSubmit = computed(() => sendBlock.value === null);

const blockingReason = computed(() => {
  switch (sendBlock.value) {
    case "incomplete_fields":
      return t("blocked_incomplete");
    case "consent_missing":
      return t("blocked_consent");
    default:
      return null;
  }
});

async function send() {
  const built = formRef.value?.validateAndBuild();
  if (!built) return;
  const ok = await submission.submit(
    { edits: {}, additions: { [built.id]: built.addition } },
    privacyChecked.value
  );
  if (ok) formRef.value?.clearPending();
}

async function cancel() {
  formRef.value?.clearPending();
  await navigateTo(localePath("/"));
}
</script>

<template>
  <div class="pt-5">
    <h1 class="text-zinc-900 dark:text-zinc-50 mb-4 text-2xl font-bold">{{ t("title") }}</h1>

    <template v-if="!submission.successUrl.value">
      <AddProposalForm ref="formRef" :initial-draft="initialDraft" embedded />

      <Toast
        v-if="submission.submitError.value"
        id="propose-error"
        class="my-4"
        :msg="submission.submitError.value"
        level="error"
      />
      <FeedbackValidationFailures class="my-4" :failures="submission.validationFailures.value" />

      <div class="border-zinc-200 dark:border-zinc-700 mt-6 border-t pt-4">
        <FeedbackConsentCheckbox v-model="privacyChecked" />
      </div>

      <div class="mt-6 flex flex-col items-end gap-2">
        <p v-if="blockingReason" class="text-amber-700 dark:text-amber-300 text-right text-xs">
          {{ blockingReason }}
        </p>
        <div class="flex flex-row-reverse gap-2">
          <FeedbackSubmitButton
            :submitting="submission.submitting.value"
            :blocked="submission.blockedByToken.value"
            :disabled="!canSubmit"
            :label="updatingEvent ? t('send_update') : undefined"
            @click="send"
          />
          <Btn variant="linkButton" size="md" @click="cancel">{{ t("cancel") }}</Btn>
        </div>
      </div>
    </template>

    <div
      v-else
      class="bg-green-50 dark:bg-green-900 border-green-200 dark:border-green-700 rounded border p-4"
    >
      <h2 class="text-green-900 dark:text-green-50 text-lg font-semibold">{{ t("thank_you") }}</h2>
      <p class="text-green-800 dark:text-green-100 mt-2 text-sm">{{ t("success_thank_you") }}</p>
      <I18nT
        tag="p"
        class="text-green-800 dark:text-green-100 mt-2 text-sm"
        keypath="success_response_at"
      >
        <template #this_pr>
          <Btn variant="link" :to="submission.successUrl.value">{{ t("success_this_pr") }}</Btn>
        </template>
      </I18nT>
      <div class="mt-4 flex gap-2">
        <Btn variant="primary" size="md" :to="localePath('/')">{{ t("back_home") }}</Btn>
      </div>
    </div>
  </div>
</template>

<i18n lang="yaml">
de:
  title: Neuen Eintrag vorschlagen
  description: Schlage einen neuen Raum, ein neues Gebäude, einen POI oder eine Veranstaltung für NavigaTUM vor.
  blocked_incomplete: Bitte fülle alle hervorgehobenen Pflichtfelder aus, um den Vorschlag zu senden.
  blocked_consent: Bitte akzeptiere die Datenschutzerklärung, um den Vorschlag zu senden.
  send_update: Aktualisierung vorschlagen
  cancel: Abbrechen
  back_home: Zur Startseite
  thank_you: Vielen Dank!
  success_thank_you: Vielen Dank für deinen Vorschlag! Wir werden ihn schnellstmöglich bearbeiten.
  success_response_at: Du findest unsere Antwort auf {this_pr}
  success_this_pr: diesem GitHub Pull Request
en:
  title: Propose a new entry
  description: Propose a new room, building, POI, or event for NavigaTUM.
  blocked_incomplete: Please complete all highlighted required fields to send the proposal.
  blocked_consent: Please accept the privacy statement to send the proposal.
  send_update: Propose update
  cancel: Cancel
  back_home: Back to home
  thank_you: Thank you!
  success_thank_you: Thank you for your proposal! We will process it as soon as possible.
  success_response_at: You can see our response at {this_pr}
  success_this_pr: this GitHub pull request
</i18n>
