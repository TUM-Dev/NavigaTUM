<template>
  <header class="shadow-black/5 dark:shadow-black/20 bg-zinc-50 fixed top-0 z-20 w-full shadow-md dark:bg-zinc-100">
    <nav class="mx-auto max-w-4xl">
      <div class="mx-3 my-2.5 flex place-content-between content-between justify-between gap-x-3 md:gap-x-10 lg:mx-10">
        <NuxtLink class="my-auto hidden lg:block print:!block" to="/">
          <img alt="Navigatum Logo" class="pt-2" width="201" height="32" src="~/assets/logos/navigatum.svg" />
        </NuxtLink>
        <div class="my-auto max-w-xl flex-grow lg:col-span-3 print:!hidden">
          <slot />
        </div>
        <div class="relative my-auto flex-shrink-0 xl:absolute xl:right-5 xl:top-3 print:!hidden">
          <PreferencesPopup />
        </div>
      </div>
    </nav>
  </header>
</template>
