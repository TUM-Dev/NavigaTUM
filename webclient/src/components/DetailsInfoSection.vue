<script setup lang="ts">
import { useDetailsStore } from "@/stores/details";
import { useGlobalStore } from "@/stores/global";

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
  <div class="modal modal-lg active" id="modal-slideshow" v-if="state.image.slideshow_open && state.data?.imgs">
    <a class="modal-overlay" :aria-label="$t('close')" @click="state.hideImageSlideshow" />
    <div class="modal-container modal-fullheight">
      <div class="modal-header">
        <button class="btn btn-clear float-right" :aria-label="$t('close')" @click="state.hideImageSlideshow" />
        <h5 class="modal-title">{{ $t("view_view.slideshow.header") }}</h5>
      </div>
      <div class="modal-body">
        <div class="content">
          <div class="carousel">
            <template v-for="(_, i) in state.data?.imgs" :key="i">
              <input
                v-if="i === state.image.shown_image_id"
                :id="`slide-${i + 1}`"
                class="carousel-locator"
                type="radio"
                name="carousel-radio"
                checked
                hidden
              />
              <input
                v-else
                :id="`slide-${i + 1}`"
                class="carousel-locator"
                type="radio"
                name="carousel-radio"
                hidden
                @click="state.showImageSlideshow(i)"
              />
            </template>

            <div class="carousel-container" v-if="state.data?.imgs">
              <figure v-for="(img, i) in state.data.imgs" class="carousel-item" :key="img.name">
                <label
                  v-if="i !== 0"
                  class="item-prev btn btn-action btn-lg"
                  :for="`slide-${i}`"
                  @click="state.showImageSlideshow(i - 1)"
                >
                  <i class="icon icon-arrow-left" />
                </label>
                <label
                  v-if="i + 1 !== (state.data?.imgs?.length || 0)"
                  class="item-next btn btn-action btn-lg"
                  :for="`slide-${i + 2}`"
                  @click="state.showImageSlideshow(i + 1)"
                >
                  <i class="icon icon-arrow-right" />
                </label>
                <div itemscope itemtype="http://schema.org/ImageObject">
                  <img
                    itemprop="contentUrl"
                    :alt="$t('view_view.slideshow.image_alt')"
                    loading="lazy"
                    :src="'/cdn/lg/' + img.name"
                    :srcset="`/cdn/sm/${img.name} 1024w,/cdn/md/${img.name} 1920w,/cdn/lg/${img.name} 3860w`"
                    sizes="100vw"
                    class="img-responsive rounded"
                  />
                  <span class="d-none" v-if="img.license.url" itemprop="license"> {{ img.license.url }}</span>
                  <span class="d-none" v-else itemprop="license"> img.license.text</span>
                  <span class="d-none" v-if="img.license.url" itemprop="author"> {{ img.author.url }}</span>
                  <span class="d-none" v-else itemprop="author"> img.author.text</span>
                </div>
              </figure>
            </div>
            <div class="carousel-nav">
              <label
                v-for="(_, i) in state.data?.imgs"
                :key="i"
                class="nav-item text-hide c-hand"
                :for="`slide-${i + 1}`"
                >{{ i + 1 }}</label
              >
            </div>
          </div>
        </div>
      </div>
      <div class="modal-footer" v-if="state.image.shown_image">
        <div class="columns">
          <div class="column col-4 col-sm-6 col-md-6 text-left">
            <h6>{{ $t("view_view.slideshow.source") }}</h6>
            <a v-if="state.image.shown_image.source.url" :href="state.image.shown_image.source.url">{{
              state.image.shown_image.source.text
            }}</a>
            <template v-else>{{ state.image.shown_image.source.text }}</template>
          </div>
          <div class="column col-4 col-sm-6 col-md-6 text-center text-md-right">
            <h6>{{ $t("view_view.slideshow.author") }}</h6>
            <a v-if="state.image.shown_image.author.url" :href="state.image.shown_image.author.url">
              {{ state.image.shown_image.author.text }}
            </a>
            <template v-else>{{ state.image.shown_image.author.text }}</template>
          </div>
          <div class="column col-4 col-sm-12 col-md-12 text-md-center mt-md-3">
            <h6>{{ $t("view_view.slideshow.license") }}</h6>
            <a v-if="state.image.shown_image.license.url" :href="state.image.shown_image.license.url">
              {{ state.image.shown_image.license.text }}
            </a>
            <template v-else>{{ state.image.shown_image.license.text }}</template>
          </div>
        </div>
      </div>
    </div>
  </div>
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

/* --- Image slideshow / showcase --- */
#modal-slideshow {
  align-items: baseline;

  & .modal-container {
    position: relative;
    top: 5em;

    & .carousel-item {
      // Disable the animation of Spectre, because it appears a bit irritating.
      // It always run if we open the image slideshow and is wrong if we go back
      // in the slideshow.
      animation: none;
      transform: translateX(0);
    }
  }
}
</style>
