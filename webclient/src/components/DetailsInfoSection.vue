<script setup lang="ts">
import { useDetailsStore } from "@/stores/details";
import { useGlobalStore } from "@/stores/global";
import DetailsImageSlideshowModal from "@/components/DetailsImageSlideshowModal.vue";

const state = useDetailsStore();
const global = useGlobalStore();
</script>

<template>
  <!-- Information section (on mobile) -->
  <div class="column col-5 col-sm-12 show-sm mobile-info-section" v-if="state.data?.props?.computed">
    <h2>Informationen</h2>
    <table class="info-table">
      <tbody>
        <tr v-for="prop in state.data.props.computed" :key="prop.name">
          <td>
            <strong>{{ prop.name }}</strong>
          </td>
          <td>
            {{ prop.text }}
            <div
              class="popover"
              v-if="prop.extra?.body"
              @click="global.information_modal = { body: prop.extra.body, header: prop.extra.header }"
            >
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
            </div>
          </td>
        </tr>
        <tr v-if="state.data.props.links">
          <td>
            <strong>{{ $t("view_view.info_table.links") }}</strong>
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
      </tbody>
    </table>
  </div>

  <!-- Informationen card (desktop) -->
  <!-- Some elements are currently duplicate, which is not optimal but should be okay
       as long as only little information is there -->
  <div class="column col-5 col-md-12 hide-sm">
    <div class="card">
      <a
        class="card-image c-hand"
        @click="state.showImageSlideshow(state.image.shown_image_id || 0)"
        v-if="state.image.shown_image"
      >
        <img
          :alt="$t('view_view.header.image_alt')"
          :src="'/cdn/header/' + state.image.shown_image.name"
          class="img-responsive"
          style="width: 100%"
        />
      </a>
      <div class="card-header">
        <div class="card-title h5">{{ $t("view_view.info_title") }}</div>
      </div>
      <div class="card-body">
        <table class="info-table" v-if="state.data?.props?.computed">
          <tbody>
            <tr v-for="prop in state.data.props.computed" :key="prop.name">
              <td>
                <strong>{{ prop.name }}</strong>
              </td>
              <td>
                {{ prop.text }}
                <div class="popover" v-if="prop.extra?.body">
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
                  <div class="popover-container">
                    <div class="card">
                      <div class="card-header" v-if="prop.extra.header">
                        {{ prop.extra.header }}
                      </div>
                      <div class="card-body">
                        {{ prop.extra.body }}
                      </div>
                      <div class="card-footer" v-if="prop.extra.footer">
                        {{ prop.extra.footer }}
                      </div>
                    </div>
                  </div>
                </div>
              </td>
            </tr>
            <tr v-if="state.data?.props.links">
              <td>
                <strong>{{ $t("view_view.info_table.links") }}</strong>
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
          </tbody>
        </table>
        <span v-else>-</span>
        <div class="toast toast-warning" v-if="state.data?.coords.accuracy === 'building'">
          {{ $t("view_view.msg.inaccurate_only_building.primary_msg") }}<br />
          <i>
            {{ $t("view_view.msg.inaccurate_only_building.help_others_and") }}
            <button class="btn btn-sm" @click="addLocationPicker">
              {{ $t("view_view.msg.inaccurate_only_building.btn") }}
            </button>
          </i>
        </div>
        <div
          class="toast toast-warning"
          v-if="state.data?.type === 'room' && state.data?.maps?.overlays?.default === null"
        >
          {{ $t("view_view.msg.no_floor_overlay") }}
        </div>
        <div class="toast" v-if="state.data?.props?.comment">
          {{ state.data.props.comment }}
        </div>
      </div>
      <!--<div class="card-footer">
          <button class="btn btn-link">Mehr Infos</button>
      </div>-->
    </div>
  </div>
  <DetailsImageSlideshowModal v-if="state.image.slideshow_open"/>
</template>

<style lang="scss">
@import "../assets/variables";
/* --- Information Card (desktop) --- */
.card-body .toast {
  margin-top: 12px;
}
/* --- Information Section (mobile) --- */
.mobile-info-section {
  margin-top: 15px;

  & > .info-table {
    margin-top: 16px;
  }
}

/* --- Info table --- */
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
