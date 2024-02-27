<script setup lang="ts">
import { useDetailsStore } from "@/stores/details";
import TinyModal from "@/components/TinyModal.vue";
import { InformationCircleIcon } from "@heroicons/vue/24/outline";
import { useI18n } from "vue-i18n";
import Btn from "@/components/Btn.vue";

const state = useDetailsStore();
const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <table class="text-zinc-600">
    <tbody>
      <tr v-for="prop in state.data?.props.computed" :key="prop.name">
        <td>
          <strong>{{ prop.name }}</strong>
        </td>
        <td>
          {{ prop.text }}
          <TinyModal v-if="prop.extra?.body" :content="prop.extra">
            <template #icon>
              <InformationCircleIcon class="h-4 w-4" />
            </template>
          </TinyModal>
        </td>
      </tr>
      <tr v-if="state.data?.props.links">
        <td>
          <strong>{{ t("links") }}</strong>
        </td>
        <td>
          <ul>
            <li v-for="link in state.data.props.links" :key="link.text">
              <Btn size="" variant="link" :to="link.url">
                {{ link.text }}
              </Btn>
            </li>
          </ul>
        </td>
      </tr>
      <tr v-if="!state.data?.props.links && !state.data?.props.computed">
        <td>
          <strong>{{ t("no_information_known") }}</strong>
        </td>
      </tr>
    </tbody>
  </table>
</template>

<i18n lang="yaml">
de:
  links: Links
  no_information_known: No information known
en:
  links: Links
  no_information_known: Keine Informationen bekannt
</i18n>
