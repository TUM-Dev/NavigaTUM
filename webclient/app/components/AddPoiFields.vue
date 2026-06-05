<script setup lang="ts">
import { mdiClose, mdiPlus } from "@mdi/js";
import { useEditProposal } from "~/composables/editProposal";

const editProposal = useEditProposal();
const { t } = useI18n({ useScope: "local" });

function addProp() {
  editProposal.value.pendingAddition.generic_props.push({ name_de: "", name_en: "", text: "" });
}
function removeProp(idx: number) {
  editProposal.value.pendingAddition.generic_props.splice(idx, 1);
}
</script>

<template>
  <div class="space-y-3">
    <div>
      <label class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium" for="add-poi-name">
        {{ t("name") }} <span class="text-red-700 dark:text-red-200">*</span>
      </label>
      <input
        id="add-poi-name"
        v-model="editProposal.pendingAddition.name"
        type="text"
        class="focusable bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-zinc-50 w-full rounded border px-2 py-1 text-sm"
      />
    </div>

    <div>
      <label class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium" for="add-poi-usage">
        {{ t("short_description") }} <span class="text-red-700 dark:text-red-200">*</span>
      </label>
      <input
        id="add-poi-usage"
        v-model="editProposal.pendingAddition.usage_name"
        type="text"
        class="focusable bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-zinc-50 w-full rounded border px-2 py-1 text-sm"
      />
      <I18nT keypath="short_description_help" tag="p" class="text-zinc-500 dark:text-zinc-400 mt-1 text-xs">
        <template #ex1><code class="font-mono">{{ t("short_description_help_ex1") }}</code></template>
        <template #ex2><code class="font-mono">{{ t("short_description_help_ex2") }}</code></template>
      </I18nT>
    </div>

    <details class="border-zinc-300 dark:border-zinc-600 rounded border px-3 py-2">
      <summary class="text-zinc-600 dark:text-zinc-300 cursor-pointer text-xs font-medium">{{ t("more_options") }}</summary>
      <p class="text-zinc-500 dark:text-zinc-400 mt-1 text-xs">{{ t("more_options_help") }}</p>
      <div class="mt-3 space-y-3">
        <div>
          <span class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium">{{ t("comment") }}</span>
          <div class="space-y-2">
            <textarea
              v-model="editProposal.pendingAddition.comment_de"
              :placeholder="t('comment_de')"
              rows="2"
              class="focusable bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-zinc-50 w-full resize-y rounded border px-2 py-1 text-sm"
            />
            <textarea
              v-model="editProposal.pendingAddition.comment_en"
              :placeholder="t('comment_en')"
              rows="2"
              class="focusable bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-zinc-50 w-full resize-y rounded border px-2 py-1 text-sm"
            />
          </div>
        </div>

        <div>
          <span class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium">{{ t("links") }}</span>
          <LinkRowEditor v-model="editProposal.pendingAddition.poi_links" />
        </div>

        <div>
          <span class="text-zinc-600 dark:text-zinc-300 mb-1 block text-xs font-medium">{{ t("generic_props") }}</span>
          <p class="text-zinc-500 dark:text-zinc-400 mb-2 text-xs">{{ t("generic_props_help") }}</p>
          <div v-for="(prop, idx) in editProposal.pendingAddition.generic_props" :key="idx" class="mb-2 flex items-start gap-2">
            <div class="flex-grow space-y-1">
              <div class="grid grid-cols-2 gap-1">
                <input
                  v-model="prop.name_de"
                  type="text"
                  :placeholder="t('prop_name_de')"
                  class="focusable bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-zinc-50 rounded border px-2 py-1 text-sm"
                />
                <input
                  v-model="prop.name_en"
                  type="text"
                  :placeholder="t('prop_name_en')"
                  class="focusable bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-zinc-50 rounded border px-2 py-1 text-sm"
                />
              </div>
              <input
                v-model="prop.text"
                type="text"
                :placeholder="t('prop_text')"
                class="focusable bg-zinc-200 dark:bg-zinc-700 border-zinc-400 dark:border-zinc-500 text-zinc-900 dark:text-zinc-50 w-full rounded border px-2 py-1 text-sm"
              />
            </div>
            <button type="button" class="focusable mt-1 rounded-sm" :title="t('remove_prop')" @click="removeProp(idx)">
              <MdiIcon :path="mdiClose" :size="16" class="text-zinc-500 dark:text-zinc-400 hover:text-zinc-700 dark:hover:text-zinc-200" />
            </button>
          </div>
          <Btn variant="secondary" size="sm" @click="addProp">
            <MdiIcon :path="mdiPlus" :size="14" /> {{ t("add_prop") }}
          </Btn>
        </div>
      </div>
    </details>
  </div>
</template>

<i18n lang="yaml">
de:
  name: Name
  short_description: Kurzbeschreibung
  short_description_help: Wird in Suchergebnissen und auf Übersichtskarten als Untertitel angezeigt - also dort, wo nur eine Zeile Platz ist (z.B. {ex1}, {ex2}).
  short_description_help_ex1: Mensa
  short_description_help_ex2: Bibliothek
  more_options: Weitere Felder
  more_options_help: Optional. Kommentar, Links und zusätzliche Eigenschaften können später ergänzt werden.
  comment: Kommentar
  comment_de: Deutsch
  comment_en: Englisch
  links: Links
  generic_props: Zusätzliche Eigenschaften
  generic_props_help: Optional. Beschriftete Werte, die auf der Detailseite angezeigt werden.
  prop_name_de: Bezeichnung (DE)
  prop_name_en: Bezeichnung (EN)
  prop_text: Wert
  add_prop: Eigenschaft hinzufügen
  remove_prop: Eigenschaft entfernen
en:
  name: Name
  short_description: Short description
  short_description_help: Shown as the subtitle in search results and on overview maps - wherever only a single line fits (e.g. {ex1}, {ex2}).
  short_description_help_ex1: Cafeteria
  short_description_help_ex2: Library
  more_options: More fields
  more_options_help: Optional. Comment, links and extra properties can be added later if needed.
  comment: Comment
  comment_de: German
  comment_en: English
  links: Links
  generic_props: Additional properties
  generic_props_help: Optional. Labelled key/value pairs shown on the detail page.
  prop_name_de: Label (DE)
  prop_name_en: Label (EN)
  prop_text: Value
  add_prop: Add property
  remove_prop: Remove property
</i18n>
