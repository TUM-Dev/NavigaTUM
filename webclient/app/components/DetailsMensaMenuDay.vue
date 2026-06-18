<script setup lang="ts">
import type { components } from "~/api_types";
import { groupDishesByCategory, type MensaPriceRole } from "~/utils/mensaMenu";

type MenuDay = components["schemas"]["MensaMenuDayResponse"];

const props = defineProps<{
  readonly day: MenuDay;
  readonly priceRole: MensaPriceRole;
}>();

const { t } = useI18n({ useScope: "local" });

const groups = computed(() => groupDishesByCategory(props.day.dishes));
// Only label categories once upstream actually splits the day into more than one; a single-group
// day reads cleaner as a plain list.
const showCategoryHeadings = computed(() => groups.value.length > 1);
</script>

<template>
  <div class="flex flex-col gap-5">
    <section
      v-for="(group, groupIndex) in groups"
      :key="group.category ?? `uncategorized-${groupIndex}`"
      class="flex flex-col gap-2"
    >
      <p
        v-if="showCategoryHeadings"
        class="text-zinc-400 dark:text-zinc-500 text-xs font-medium"
      >
        {{ group.category ?? t("other") }}
      </p>
      <ul class="flex flex-col gap-3.5">
        <DetailsMensaMenuDish
          v-for="(dish, dishIndex) in group.dishes"
          :key="dishIndex"
          :dish="dish"
          :price-role="priceRole"
        />
      </ul>
    </section>
  </div>
</template>

<i18n lang="yaml">
de:
  other: Weitere Gerichte
en:
  other: More dishes
</i18n>
