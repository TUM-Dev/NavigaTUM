<script setup lang="ts">
import { useDetailsStore } from "@/stores/details";
import TinyModal from "@/components/TinyModal.vue";
import { InformationCircleIcon } from "@heroicons/vue/24/outline";
import { useI18n } from "vue-i18n";

const state = useDetailsStore();
const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <table class="info-table">
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
              <a :href="link.url">
                {{ link.text }}
              </a>
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

<style lang="scss">
@import "@/assets/variables";
.info-table {
  width: 100%;
  border-collapse: collapse;

  td {
    vertical-align: top;
    padding: 4px 0;

    &:last-child {
      padding-left: 10px;
    }

    .popover {
      .card {
        box-shadow: 0 0 6px rgba(106, 106, 106, 0.08);
        border: 0.05rem solid #e1e1e1;

        .card-header {
          font-weight: bold;
        }
      }

      svg {
        margin-left: 5px;
        margin-bottom: -2px;
      }
    }
  }

  tr {
    border-bottom: 1px solid $border-light;

    &:last-child {
      border-bottom: 0;
    }
  }

  ul {
    list-style-type: none;
    margin: 0;
  }

  li {
    margin: 0 0 0.4rem;

    &:last-child {
      margin: 0;
    }
  }
}
</style>

<i18n lang="yaml">
de:
  links: Links
  no_information_known: No information known
en:
  links: Links
  no_information_known: Keine Informationen bekannt
</i18n>
