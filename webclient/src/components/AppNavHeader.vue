<template>
  <header class="fixed navbar top-0 w-full z-10">
    <div class="container grid-lg">
      <div class="columns">
        <div class="column hide-lg">
          <RouterLink to="/">
            <img id="logo" alt="Navigatum Logo" src="@/assets/logos/navigatum.svg" />
            <!-- 7px for logo1 -->
          </RouterLink>
        </div>
        <div class="col-8 col-lg-11 col-mx-auto column">
          <slot />
        </div>
      </div>
    </div>
  </header>
</template>

<style lang="scss">
@import "@/assets/variables";

.navbar {
  padding: 10px 0;
  box-shadow: 0 2px 3px $header-shadow-color;
  background: $header-color;

  #logo {
    height: 24px;
    margin-top: 9px;
  }

  .input-group button {
    border: 0;
  }
}
</style>
