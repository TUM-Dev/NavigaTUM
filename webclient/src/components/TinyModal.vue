<script setup lang="ts">
import { useToggle, useBreakpoints } from "@vueuse/core";
import { useI18n } from "vue-i18n";

const props = defineProps<{
  content: {
    title?: string;
    body?: string;
    footer?: string;
  };
}>();

const { t } = useI18n({ useScope: "local" });
const breakpoints = useBreakpoints({ xs: 0, sm: 601, md: 841, lg: 961, xl: 1281 });
const showPopoverInstead = breakpoints.greaterOrEqual("md");
const [modalOpen, toggleModal] = useToggle(false);
</script>

<template>
  <div v-if="showPopoverInstead" class="popover">
    <slot name="icon" />
    <div class="popover-container">
      <div class="card">
        <div v-if="props.content.title" class="card-header">
          {{ props.content.title }}
        </div>
        <div v-if="props.content.body" class="card-body">
          {{ props.content.body }}
        </div>
        <div v-if="props.content.footer" class="card-footer">
          {{ props.content.footer }}
        </div>
      </div>
    </div>
  </div>
  <template v-else>
    <a class="cursor-pointer" :aria-label="t('show_more_information')" @click="toggleModal()">
      <slot name="icon" />
    </a>
    <Teleport v-if="modalOpen" to="body">
      <div class="active modal">
        <a class="modal-overlay" :aria-label="t('close')" @click="toggleModal()" />
        <div class="modal-container">
          <div class="modal-header">
            <button type="button" class="btn btn-clear float-right" :aria-label="t('close')" @click="toggleModal()" />
            <div v-if="props.content.title" class="h5 modal-title">{{ props.content.title }}</div>
          </div>
          <div class="modal-body">
            <div class="content">
              <p v-if="props.content.body">{{ props.content.body }}</p>
              <p v-if="props.content.footer">{{ props.content.footer }}</p>
            </div>
          </div>
        </div>
      </div>
    </Teleport>
  </template>
</template>

<i18n lang="yaml">
de:
  show_more_information: Mehr Informationen anzeigen
  close: Schließen
en:
  show_more_information: Show more information
  close: Close
</i18n>
