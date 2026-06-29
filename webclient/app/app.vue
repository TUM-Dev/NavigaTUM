<template>
  <NuxtRouteAnnouncer />
  <NuxtLayout>
    <NuxtPage />
  </NuxtLayout>
  <ClientOnly>
    <LazyHiringPopup />
  </ClientOnly>
</template>
