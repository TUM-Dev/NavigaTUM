<script setup lang="ts">
import { useEditProposal } from "~/composables/editProposal";

const editProposal = useEditProposal();
const { t } = useI18n({ useScope: "local" });

// Commit, commit-with-image and cancel all hand control back to the Propose Changes modal.
// Submission, privacy, and send all live there.
function handleCommit() {
  editProposal.value.addOpen = false;
  editProposal.value.open = true;
}

function handleCommitWithImage() {
  editProposal.value.addOpen = false;
  editProposal.value.open = true;
  editProposal.value.imageUpload.open = true;
}

function handleCancel() {
  editProposal.value.addOpen = false;
  editProposal.value.open = true;
}
</script>

<template>
  <Modal v-model="editProposal.addOpen" :title="t('title')" @close="handleCancel">
    <AddProposalForm
      @commit="handleCommit"
      @commit-with-image="handleCommitWithImage"
      @cancel="handleCancel"
    />
  </Modal>
</template>

<i18n lang="yaml">
de:
  title: Neuen Eintrag vorschlagen
en:
  title: Propose a new entry
</i18n>
