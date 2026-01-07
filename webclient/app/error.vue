<script setup lang="ts">
const props = defineProps({
  error: Object,
});

const is404 = computed(() => props.error?.statusCode === 404);
</script>

<template>
  <NuxtLayout>
    <NotFound v-if="is404" :path="error?.url" />
    <div v-else class="mx-auto max-w-xl pt-4">
      <div class="flex flex-col items-center gap-4 p-5">
        <h5 class="text-zinc-800 text-xl font-bold">{{ error?.statusCode || "Error" }}</h5>
        <p class="text-md text-zinc-600">{{ error?.statusMessage || "An error occurred" }}</p>
        <p v-if="error?.message" class="text-sm text-zinc-500 mt-2">
          {{ error.message }}
        </p>
        <Btn @click="clearError({ redirect: '/' })" variant="primary" class="mt-4"> Go home </Btn>
      </div>
    </div>
  </NuxtLayout>
</template>
