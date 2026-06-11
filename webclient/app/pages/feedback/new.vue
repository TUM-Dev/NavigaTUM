<script setup lang="ts">
import type { LocationQueryRaw } from "vue-router";
import {
  type Addition,
  type AdditionDraft,
  type AdditionKind,
  additionRegistry,
  emptyAdditionDraft,
} from "~/composables/additionSchema";
import { useFeedbackSubmission } from "~/composables/feedbackSubmission";
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

const KINDS = ["room", "building", "poi", "event"] as const satisfies readonly AdditionKind[];

function parseKind(value: unknown): AdditionKind | null {
  if (typeof value !== "string") return null;
  return (KINDS as readonly string[]).includes(value) ? (value as AdditionKind) : null;
}

function parsePositiveInt(value: unknown): number | null {
  if (typeof value !== "string") return null;
  const n = Number.parseInt(value, 10);
  return Number.isFinite(n) && n > 0 ? n : null;
}

// Seed the form from the URL query so a shared link like
// `/feedback/new?kind=event&org=123` pre-selects the event tab and the
// organisation without any user interaction.
function draftFromQuery(): AdditionDraft {
  const kind = parseKind(route.query.kind);
  if (!kind) return emptyAdditionDraft();
  if (kind === "event") {
    const draft = additionRegistry.event.empty();
    const orgId = parsePositiveInt(route.query.org);
    if (orgId !== null) draft.organising_org_id = orgId;
    return draft;
  }
  return additionRegistry[kind].empty();
}

// Snapshot the query once at setup time.
// Subsequent URL changes on this page are emitted by us as the user edits.
const initialDraft = draftFromQuery();

// When the URL pins an organisation, block setup on the orgs CDN so the
// Combobox resolves the selection on first render. Other kinds and event
// without `?org=` skip this and get a non-blocking lazy fetch as before.
if (initialDraft.kind === "event" && initialDraft.organising_org_id !== null) {
  await useKnownOrgs().ready();
}

// router.replace fires inside watch callbacks during the unmount lifecycle
// would race with the outgoing navigation, so gate writes on a mounted flag.
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
    if (orgId !== null) next.org = String(orgId);
    else delete next.org;
    if (next.kind === route.query.kind && next.org === route.query.org) return;
    router.replace({ query: next });
  }
);

const formRef = ref<{
  validateAndBuild(): {
    id: string;
    displayName: string;
    addition: NonNullable<Addition>;
  } | null;
  clearPending(): void;
  draftIsReady: { value: boolean };
} | null>(null);

const privacyChecked = ref(false);

const canSubmit = computed(() => {
  if (submission.submitting.value || submission.successUrl.value) return false;
  if (submission.blockedByToken.value) return false;
  if (!privacyChecked.value) return false;
  return formRef.value?.draftIsReady.value === true;
});

async function send() {
  const built = formRef.value?.validateAndBuild();
  if (!built) return; // the form surfaces its own draft-level error.
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
        id="feedback-new-error"
        class="my-4"
        :msg="submission.submitError.value"
        level="error"
      />
      <FeedbackValidationFailures class="my-4" :failures="submission.validationFailures.value" />

      <div class="border-zinc-200 dark:border-zinc-700 mt-6 border-t pt-4">
        <FeedbackConsentCheckbox v-model="privacyChecked" />
      </div>

      <div class="float-right mt-6 flex flex-row-reverse gap-2">
        <FeedbackSubmitButton
          :submitting="submission.submitting.value"
          :blocked="submission.blockedByToken.value"
          :disabled="!canSubmit"
          @click="send"
        />
        <Btn variant="linkButton" size="md" @click="cancel">{{ t("cancel") }}</Btn>
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
  cancel: Abbrechen
  back_home: Zur Startseite
  thank_you: Vielen Dank!
  success_thank_you: Vielen Dank für deinen Vorschlag! Wir werden ihn schnellstmöglich bearbeiten.
  success_response_at: Du findest unsere Antwort auf {this_pr}
  success_this_pr: diesem GitHub Pull Request
en:
  title: Propose a new entry
  description: Propose a new room, building, POI, or event for NavigaTUM.
  cancel: Cancel
  back_home: Back to home
  thank_you: Thank you!
  success_thank_you: Thank you for your proposal! We will process it as soon as possible.
  success_response_at: You can see our response at {this_pr}
  success_this_pr: this GitHub pull request
</i18n>
