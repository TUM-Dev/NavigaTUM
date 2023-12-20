<script setup lang="ts">
import { useDetailsStore } from "@/stores/details";
import { useI18n } from "vue-i18n";
import Modal from "@/components/Modal.vue";
import { ChevronLeftIcon, ChevronRightIcon } from "@heroicons/vue/24/outline";

const state = useDetailsStore();
const { t } = useI18n({ useScope: "local" });
const appURL = import.meta.env.VITE_APP_URL;
</script>

<template>
  <Modal v-if="state.data?.imgs" v-model:open="state.image.slideshow_open" :title="t('header')">
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
        <figure v-for="(img, i) in state.data.imgs" :key="img.name" class="carousel-item !transform-none !animate-none">
          <label
            v-if="i !== 0"
            class="btn btn-action btn-lg item-prev"
            :for="`slide-${i}`"
            @click="state.showImageSlideshow(i - 1)"
          >
            <ChevronLeftIcon class="h-4 w-4" />
          </label>
          <label
            v-if="i + 1 !== state.data.imgs.length"
            class="btn btn-action btn-lg item-next"
            :for="`slide-${i + 2}`"
            @click="state.showImageSlideshow(i + 1)"
          >
            <ChevronRightIcon class="h-4 w-4" />
          </label>
          <div itemscope itemtype="http://schema.org/ImageObject">
            <img
              itemprop="contentUrl"
              :alt="t('image_alt')"
              loading="lazy"
              :src="`${appURL}/cdn/lg/${img.name}`"
              :srcset="`${appURL}/cdn/sm/${img.name} 1024w,${appURL}/cdn/md/${img.name} 1920w,${appURL}/cdn/lg/${img.name} 3860w`"
              sizes="100vw"
              class="block h-auto max-w-full rounded bg-zinc-100"
            />
            <span v-if="img.license.url" class="hidden" itemprop="license"> {{ img.license.url }}</span>
            <span v-else class="hidden" itemprop="license"> img.license.text</span>
            <span v-if="img.license.url" class="hidden" itemprop="author"> {{ img.author.url }}</span>
            <span v-else class="hidden" itemprop="author"> img.author.text</span>
          </div>
        </figure>
      </div>
      <div class="carousel-nav">
        <label
          v-for="(_, i) in state.data.imgs"
          :key="i"
          class="nav-item text-hide cursor-pointer"
          :for="`slide-${i + 1}`"
          >{{ i + 1 }}</label
        >
      </div>
    </div>
    <div v-if="state.image.shown_image" class="mt-5">
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
