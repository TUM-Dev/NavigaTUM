<script setup lang="ts">
import { useDetailsStore } from "@/stores/details";
import { useI18n } from "vue-i18n";

const state = useDetailsStore();
const { t } = useI18n({ useScope: "local" });
const appURL = import.meta.env.VITE_APP_URL;
</script>

<template>
  <Teleport v-if="state.data?.imgs" to="body">
    <div id="modal-slideshow" class="active modal modal-lg">
      <a class="modal-overlay" :aria-label="t('close')" @click="state.hideImageSlideshow" />
      <div class="modal-container modal-fullheight">
        <div class="modal-header">
          <button
            type="button"
            class="btn btn-clear float-right"
            :aria-label="t('close')"
            @click="state.hideImageSlideshow"
          />
          <h5 class="modal-title">{{ t("header") }}</h5>
        </div>
        <div class="modal-body">
          <div class="content">
            <div class="carousel">
              <template v-for="(_, i) in state.data.imgs" :key="i">
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

              <div class="carousel-container">
                <figure v-for="(img, i) in state.data.imgs" :key="img.name" class="carousel-item">
                  <label
                    v-if="i !== 0"
                    class="btn btn-action btn-lg item-prev"
                    :for="`slide-${i}`"
                    @click="state.showImageSlideshow(i - 1)"
                  >
                    <i class="icon icon-arrow-left" />
                  </label>
                  <label
                    v-if="i + 1 !== state.data.imgs.length"
                    class="btn btn-action btn-lg item-next"
                    :for="`slide-${i + 2}`"
                    @click="state.showImageSlideshow(i + 1)"
                  >
                    <i class="icon icon-arrow-right" />
                  </label>
                  <div itemscope itemtype="http://schema.org/ImageObject">
                    <img
                      itemprop="contentUrl"
                      :alt="t('image_alt')"
                      loading="lazy"
                      :src="`${appURL}/cdn/lg/${img.name}`"
                      :srcset="`${appURL}/cdn/sm/${img.name} 1024w,${appURL}/cdn/md/${img.name} 1920w,${appURL}/cdn/lg/${img.name} 3860w`"
                      sizes="100vw"
                      class="block h-auto max-w-full bg-zinc-100 rounded"
                    />
                    <span v-if="img.license.url" class="d-none" itemprop="license"> {{ img.license.url }}</span>
                    <span v-else class="d-none" itemprop="license"> img.license.text</span>
                    <span v-if="img.license.url" class="d-none" itemprop="author"> {{ img.author.url }}</span>
                    <span v-else class="d-none" itemprop="author"> img.author.text</span>
                  </div>
                </figure>
              </div>
              <div class="carousel-nav">
                <label
                  v-for="(_, i) in state.data.imgs"
                  :key="i"
                  class="c-hand nav-item text-hide"
                  :for="`slide-${i + 1}`"
                  >{{ i + 1 }}</label
                >
              </div>
            </div>
          </div>
        </div>
        <div v-if="state.image.shown_image" class="modal-footer">
          <div class="columns">
            <div class="col-4 col-md-6 col-sm-6 column text-left">
              <h6>{{ t("source") }}</h6>
              <a v-if="state.image.shown_image.source.url" :href="state.image.shown_image.source.url">
                {{ state.image.shown_image.source.text }}
              </a>
              <template v-else>{{ state.image.shown_image.source.text }}</template>
            </div>
            <div class="col-4 col-md-6 col-sm-6 column text-center text-md-right">
              <h6>{{ t("author") }}</h6>
              <a v-if="state.image.shown_image.author.url" :href="state.image.shown_image.author.url">
                {{ state.image.shown_image.author.text }}
              </a>
              <template v-else>{{ state.image.shown_image.author.text }}</template>
            </div>
            <div class="col-4 col-md-12 col-sm-12 column mt-md-3 text-md-center">
              <h6>{{ t("license") }}</h6>
              <a v-if="state.image.shown_image.license.url" :href="state.image.shown_image.license.url">
                {{ state.image.shown_image.license.text }}
              </a>
              <template v-else>{{ state.image.shown_image.license.text }}</template>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style lang="scss">
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

<i18n lang="yaml">
de:
  close: Schließen
  author: Autor
  header: Bilder-Showcase
  image_alt: Ein Bild welches das Gebäude zeigt
  license: Lizenz
  source: Quelle
en:
  close: Close
  author: Author
  header: Image Showcase
  image_alt: Image showing the building
  license: License
  source: Source
</i18n>
