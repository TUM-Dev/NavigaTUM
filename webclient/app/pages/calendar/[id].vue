<script setup lang="ts">
onMounted(async () => {
  const route = useRoute();
  const localePath = useLocalePath();
  await navigateTo({
    path: localePath(`/view/${route.params.id}`),
    query: { "calendar[]": route.params.id },
    replace: true,
  });
});
</script>
