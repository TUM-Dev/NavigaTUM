<script setup lang="ts">
import { useDetailsStore } from "@/stores/details";

const state = useDetailsStore();
</script>

<template>
  <!-- Information section (on mobile) -->
  <div
    class="column col-5 col-sm-12 show-sm mobile-info-section"
    v-if="state.data.props?.computed"
  >
    <h2>Informationen</h2>
    <table class="info-table">
      <tbody>
        <tr v-for="prop in state.data.props.computed">
          <td>
            <strong>{{ prop.name }}</strong>
          </td>
          <td>{{ prop.text }}</td>
        </tr>
        <tr v-if="state.data.props.links">
          <td>
            <strong>{{ $t("view_view.info_table.links") }}</strong>
          </td>
          <td>
            <ul>
              <li v-for="link in state.data.props.links">
                <a v-bind:href="link.url">
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
        @click="state.showImageSlideshow(state.image.shown_image_id)"
        v-if="state.image.shown_image"
      >
        <img
          alt="Header-Image, showing the building"
          v-bind:src="'/cdn/header/' + state.image.shown_image.name"
          class="img-responsive"
          width="100%"
        />
      </a>
      <div class="card-header">
        <div class="card-title h5">{{ $t("view_view.info_title") }}</div>
      </div>
      <div class="card-body">
        <table class="info-table" v-if="state.data.props?.computed">
          <tbody>
            <tr v-for="prop in state.data.props.computed">
              <td>
                <strong>{{ prop.name }}</strong>
              </td>
              <td>{{ prop.text }}</td>
            </tr>
            <tr v-if="state.data.props.links">
              <td>
                <strong>{{ $t("view_view.info_table.links") }}</strong>
              </td>
              <td>
                <ul>
                  <li v-for="link in state.data.props.links">
                    <a v-bind:href="link.url">
                      {{ link.text }}
                    </a>
                  </li>
                </ul>
              </td>
            </tr>
          </tbody>
        </table>
        <span v-else>-</span>
        <div
          class="toast toast-warning"
          v-if="state.data.coords.accuracy === 'building'"
        >
          {{ $t("view_view.msg.inaccurate_only_building.msg") }}
          <button class="btn btn-sm" @click="addLocationPicker">
            {{ $t("view_view.msg.inaccurate_only_building.btn") }}
          </button>
        </div>
        <div
          class="toast toast-warning"
          v-if="
            state.data.type === 'room' &&
            state.data.maps?.overlays?.default === null
          "
        >
          {{ $t("view_view.msg.no_floor_overlay") }}
        </div>
        <div class="toast" v-if="state.data.props?.comment">
          {{ state.data.props.comment }}
        </div>
      </div>
      <!--<div class="card-footer">
          <button class="btn btn-link">Mehr Infos</button>
      </div>-->
    </div>
  </div>
  <div
    class="modal modal-lg active"
    id="modal-slideshow"
    v-if="state.image.slideshow_open"
  >
    <a
      class="modal-overlay"
      aria-label="Close"
      @click="state.hideImageSlideshow"
    ></a>
    <div class="modal-container modal-fullheight">
      <div class="modal-header">
        <button
          class="btn btn-clear float-right"
          v-bind:aria-label="$t('view_view.slideshow.close')"
          @click="state.hideImageSlideshow"
        ></button>
        <h5 class="modal-title">{{ $t("view_view.slideshow.header") }}</h5>
      </div>
      <div class="modal-body">
        <div class="content">
          <div class="carousel">
            <template v-for="(_, i) in state.data.imgs">
              <input
                v-if="i === state.image.shown_image_id"
                v-bind:id="'slide-' + (i + 1)"
                class="carousel-locator"
                type="radio"
                name="carousel-radio"
                hidden=""
                checked="checked"
              />
              <input
                v-else
                v-bind:id="'slide-' + (i + 1)"
                class="carousel-locator"
                type="radio"
                name="carousel-radio"
                hidden=""
                @click="state.showImageSlideshow(i)"
              />
            </template>

            <div class="carousel-container">
              <figure v-for="(img, i) in state.data.imgs" class="carousel-item">
                <label
                  v-if="i !== 0"
                  class="item-prev btn btn-action btn-lg"
                  v-bind:for="'slide-' + i"
                  @click="state.showImageSlideshow(i - 1)"
                >
                  <i class="icon icon-arrow-left"></i>
                </label>
                <label
                  v-if="i !== state.data.imgs.length - 1"
                  class="item-next btn btn-action btn-lg"
                  v-bind:for="'slide-' + (i + 2)"
                  @click="state.showImageSlideshow(i + 1)"
                >
                  <i class="icon icon-arrow-right"></i>
                </label>
                <div itemscope itemtype="http://schema.org/ImageObject">
                  <img
                    itemprop="contentUrl"
                    v-bind:alt="$t('view_view.slideshow.image_alt')"
                    loading="lazy"
                    v-bind:src="'/cdn/lg/' + img.name"
                    v-bind:srcset="
                      '/cdn/sm/' +
                      img.name +
                      ' 1024w,' +
                      '/cdn/md/' +
                      img.name +
                      ' 1920w,' +
                      '/cdn/lg/' +
                      img.name +
                      ' 3860w'
                    "
                    sizes="100vw"
                    class="img-responsive rounded"
                  />
                  <span
                    class="d-none"
                    v-if="img.license.url"
                    itemprop="license"
                  >
                    {{ img.license.url }}</span
                  >
                  <span class="d-none" v-else itemprop="license">
                    img.license.text</span
                  >
                  <span class="d-none" v-if="img.license.url" itemprop="author">
                    {{ img.author.url }}</span
                  >
                  <span class="d-none" v-else itemprop="author">
                    img.author.text</span
                  >
                </div>
              </figure>
            </div>
            <div class="carousel-nav">
              <label
                v-for="(_, i) in state.data.imgs"
                class="nav-item text-hide c-hand"
                v-bind:for="'slide-' + (i + 1)"
                >{{ i + 1 }}</label
              >
            </div>
          </div>
        </div>
      </div>
      <div class="modal-footer">
        <div class="columns">
          <div class="column col-4 col-sm-6 col-md-6 text-left">
            <h6>{{ $t("view_view.slideshow.source") }}</h6>
            <a
              v-if="state.image.shown_image.source.url"
              v-bind:href="state.image.shown_image.source.url"
              >{{ state.image.shown_image.source.text }}</a
            >
            <template v-else>{{
              state.image.shown_image.source.text
            }}</template>
          </div>
          <div class="column col-4 col-sm-6 col-md-6 text-center text-md-right">
            <h6>{{ $t("view_view.slideshow.author") }}</h6>
            <a
              v-if="state.image.shown_image.author.url"
              v-bind:href="state.image.shown_image.author.url"
              >{{ state.image.shown_image.author.text }}</a
            >
            <template v-else>{{
              state.image.shown_image.author.text
            }}</template>
          </div>
          <div class="column col-4 col-sm-12 col-md-12 text-md-center mt-md-3">
            <h6>{{ $t("view_view.slideshow.license") }}</h6>
            <a
              v-if="state.image.shown_image.license.url"
              v-bind:href="state.image.shown_image.license.url"
              >{{ state.image.shown_image.license.text }}</a
            >
            <template v-else>{{
              state.image.shown_image.license.text
            }}</template>
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
