<script setup lang="ts">
import { useDetailsStore } from "@/stores/details";
import TinyModal from "@/components/TinyModal.vue";
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
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="16"
                height="16"
                fill="currentColor"
                class="bi bi-info-circle"
                viewBox="0 0 16 16"
              >
                <path d="M8 15A7 7 0 1 1 8 1a7 7 0 0 1 0 14zm0 1A8 8 0 1 0 8 0a8 8 0 0 0 0 16z" />
                <path
                  d="m8.93 6.588-2.29.287-.082.38.45.083c.294.07.352.176.288.469l-.738 3.468c-.194.897.105 1.319.808 1.319.545 0 1.178-.252 1.465-.598l.088-.416c-.2.176-.492.246-.686.246-.275 0-.375-.193-.304-.533L8.93 6.588zM9 4.5a1 1 0 1 1-2 0 1 1 0 0 1 2 0z"
                />
              </svg>
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
