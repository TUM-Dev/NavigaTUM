<template>
  <header class="navbar">
    <div class="container grid-lg">
      <div class="columns">
        <div class="column hide-lg">
          <RouterLink to="/">
            <img :alt="$t('meta.logo_alt')" src="@/assets/logos/navigatum.svg" id="logo" />
            <!-- 7px for logo1 -->
          </RouterLink>
        </div>
        <div class="column col-8 col-lg-11 col-mx-auto">
          <slot />
        </div>

        <div class="hide-lg">
          <RouterLink to="/">
            <img :alt="$t('meta.tum_logo_alt')" src="../assets/logos/tum.svg" id="tum_logo" />
            <!-- 7px for logo1 -->
          </RouterLink>
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
  width: 100%;
  position: fixed;
  background: $header-color;
  top: 0;
  z-index: 2000;

  #logo {
    height: 24px;
    margin-top: 9px;
  }

  #tum_logo {
    position: absolute;
    margin: 20px;
    right: 0;
    top: 0;
    height: 20px;
  }

  @media (max-width: 1130px) {
    #tum_logo {
      position: relative;
      margin: 10px 10px 0 10px;
    }
  }

  .input-group button {
    border: 0;
  }
}
</style>
