<script setup lang="ts">
import type { LocationQueryRaw } from "vue-router";
import {
  type AdditionDraft,
  type AdditionKind,
  additionRegistry,
  emptyAdditionDraft,
} from "~/composables/additionSchema";
import { useEditProposal } from "~/composables/editProposal";
import { useKnownOrgs } from "~/composables/useKnownOrgs";

const { t } = useI18n({ useScope: "local" });
const editProposal = useEditProposal();
const localePath = useLocalePath();
const route = useRoute();
const router = useRouter();

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

// Mirror the modal's post-commit flow: hand off to the Propose Changes modal
// where the privacy checkbox and send action live.
function handleCommit() {
  editProposal.value.open = true;
}

function handleCommitWithImage() {
  editProposal.value.open = true;
  editProposal.value.imageUpload.open = true;
}

async function handleCancel() {
  await navigateTo(localePath("/"));
}
</script>

<template>
  <div class="pt-5">
    <h1 class="text-zinc-900 dark:text-zinc-50 mb-4 text-2xl font-bold">{{ t("title") }}</h1>
    <AddProposalForm
      :initial-draft="initialDraft"
      @commit="handleCommit"
      @commit-with-image="handleCommitWithImage"
      @cancel="handleCancel"
    />
    <ClientOnly>
      <LazyEditProposalModal v-if="editProposal.open" />
    </ClientOnly>
  </div>
</template>

<i18n lang="yaml">
de:
  title: Neuen Eintrag vorschlagen
  description: Schlage einen neuen Raum, ein neues Gebäude, einen POI oder eine Veranstaltung für NavigaTUM vor.
en:
  title: Propose a new entry
  description: Propose a new room, building, POI, or event for NavigaTUM.
</i18n>
