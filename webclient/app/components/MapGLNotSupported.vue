<script setup lang="ts">
const { t } = useI18n({ useScope: "local" });
</script>

<template>
  <div class="w-full h-full flex flex-col items-center justify-center text-center p-3 text-lg leading-normal bg-red-100 text-red-950">
    <p class="hidden md:block mb-4">
      {{ t("no_webgl.detailed_message") }}
    </p>
    <p>
      <a
        href="https://wiki.openstreetmap.org/wiki/This_map_requires_WebGL"
        target="_blank"
        rel="noopener noreferrer"
        class="text-inherit font-bold hover:underline inline-flex flex-col items-center"
      >
        <span>{{ t("no_webgl.webgl_required") }}</span>
        <small class="text-sm">{{ t("no_webgl.read_more") }}</small>
      </a>
    </p>
  </div>
</template>

<i18n lang="yaml">
de:
  no_webgl:
    detailed_message: Es tut uns leid, aber es scheint, dass Ihr Browser WebGL nicht unterstützt, eine Technologie zum Rendern von 3D-Grafiken im Web.
    webgl_required: WebGL ist erforderlich, um diese Karte anzuzeigen.
    read_more: Mehr erfahren
en:
  no_webgl:
    detailed_message: We are sorry, but it seems that your browser does not support WebGL, a technology for rendering 3D graphics on the web.
    webgl_required: WebGL is required to display this map.
    read_more: Read more
</i18n>
