<script setup lang="ts">
import { mdiClose, mdiPlus } from "@mdi/js";
import type { LinkDraft } from "~/composables/additionSchema";

const links = defineModel<LinkDraft[]>({ required: true });
const { t } = useI18n({ useScope: "local" });

function addLink() {
  links.value.push({ text_de: "", text_en: "", url: "" });
}
function removeLink(idx: number) {
  links.value.splice(idx, 1);
}
</script>

<template>
  <div>
    <div v-for="(link, idx) in links" :key="idx" class="mb-2 flex items-start gap-2">
      <div class="flex-grow space-y-1">
        <input
          v-model="link.url"
          type="url"
          placeholder="https://"
          class="focusable input-field w-full rounded border px-2 py-1 text-sm"
        />
        <div class="grid grid-cols-2 gap-1">
          <input
            v-model="link.text_de"
            type="text"
            :placeholder="t('text_de')"
            class="focusable input-field rounded border px-2 py-1 text-sm"
          />
          <input
            v-model="link.text_en"
            type="text"
            :placeholder="t('text_en')"
            class="focusable input-field rounded border px-2 py-1 text-sm"
          />
        </div>
      </div>
      <button type="button" class="focusable mt-1 rounded-sm" :title="t('remove')" @click="removeLink(idx)">
        <MdiIcon :path="mdiClose" :size="16" class="text-zinc-500 dark:text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-200" />
      </button>
    </div>
    <Btn variant="secondary" size="sm" @click="addLink">
      <MdiIcon :path="mdiPlus" :size="14" /> {{ t("add") }}
    </Btn>
  </div>
</template>

<i18n lang="yaml">
de:
  text_de: Text (DE)
  text_en: Text (EN)
  add: Link hinzufügen
  remove: Link entfernen
en:
  text_de: Text (DE)
  text_en: Text (EN)
  add: Add link
  remove: Remove link
</i18n>
