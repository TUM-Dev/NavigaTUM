<script setup lang="ts">
import "vue3-carousel/dist/carousel.css";
import { Carousel, Slide, Pagination, Navigation } from "vue3-carousel";
import { useDetailsStore } from "@/stores/details";
import { useI18n } from "vue-i18n";
import Modal from "@/components/Modal.vue";

const state = useDetailsStore();
const { t } = useI18n({ useScope: "local" });
const appURL = import.meta.env.VITE_APP_URL;

type OnSlideData = {
  currentSlideIndex: number;
  prevSlideIndex: number;
  slidesCount: number;
};
function onSlide({ currentSlideIndex }: OnSlideData): void {
  // if-statement just to make ts happy
  if (state.data?.imgs) state.image.shown_image = state.data?.imgs[currentSlideIndex];
}
</script>

<template>
  <Modal
    v-if="state.data?.imgs"
    v-model="state.image.slideshow_open"
    :title="t('header')"
    :classes="{ modal: '!min-w-[60vw]' }"
  >
    <Carousel
      :items-to-show="1.15"
      snap-align="center"
      :autoplay="10_000"
      :pause-autoplay-on-hover="true"
      @slide-end="onSlide"
    >
      <Slide v-for="img in state.data.imgs" :key="img.name">
        <div itemscope itemtype="http://schema.org/ImageObject" class="px-2">
          <img
            itemprop="contentUrl"
            :alt="t('image_alt')"
            :src="`${appURL}/cdn/lg/${img.name}`"
            :srcset="`${appURL}/cdn/sm/${img.name} 1024w,${appURL}/cdn/md/${img.name} 1920w,${appURL}/cdn/lg/${img.name} 3860w`"
            sizes="100vw"
            class="h-full w-full rounded"
          />
          <span v-if="img.license.url" class="hidden" itemprop="license"> {{ img.license.url }}</span>
          <span v-else class="hidden" itemprop="license"> img.license.text</span>
          <span v-if="img.license.url" class="hidden" itemprop="author"> {{ img.author.url }}</span>
          <span v-else class="hidden" itemprop="author"> img.author.text</span>
        </div>
      </Slide>
      <template #addons>
        <Navigation />
        <Pagination />
      </template>
    </Carousel>
    <div v-if="state.image.shown_image" class="pt-5">
      <div class="grid grid-cols-3 gap-5 text-center">
        <div class="col-span-3 md:col-span-1 md:text-left">
          <h6>{{ t("source") }}</h6>
          <a v-if="state.image.shown_image.source.url" :href="state.image.shown_image.source.url">
            {{ state.image.shown_image.source.text }}
          </a>
          <template v-else>{{ state.image.shown_image.source.text }}</template>
        </div>
        <div class="col-span-3 md:col-span-1">
          <h6>{{ t("author") }}</h6>
          <a v-if="state.image.shown_image.author.url" :href="state.image.shown_image.author.url">
            {{ state.image.shown_image.author.text }}
          </a>
          <template v-else>{{ state.image.shown_image.author.text }}</template>
        </div>
        <div class="col-span-3 md:col-span-1 md:!text-right">
          <h6>{{ t("license") }}</h6>
          <a v-if="state.image.shown_image.license.url" :href="state.image.shown_image.license.url">
            {{ state.image.shown_image.license.text }}
          </a>
          <template v-else>{{ state.image.shown_image.license.text }}</template>
        </div>
      </div>
    </div>
  </Modal>
</template>

<i18n lang="yaml">
de:
  author: Autor
  header: Bilder-Showcase
  image_alt: Ein Bild welches das Gebäude zeigt
  license: Lizenz
  source: Quelle
en:
  author: Author
  header: Image Showcase
  image_alt: Image showing the building
  license: License
  source: Source
</i18n>
