<script setup lang="ts"></script>

<template>
  <div v-if="state.data.sections?.featured">
    <div class="columns" style="margin-top: 40px">
      <div class="column"><h2>Featured</h2></div>
    </div>
    <div style="position: relative; overflow: hidden; white-space: nowrap">
      <div
        style="
          position: absolute;
          height: 100%;
          display: flex;
          background: linear-gradient(-90deg, #fff0 0%, #fffd 100%);
        "
      >
        <button class="btn btn-action s-circle" style="margin: auto 0; /*! position: absolute; */">
          <i class="icon icon-arrow-left"></i>
        </button>
      </div>
      <div class="card" style="width: 250px; display: inline-flex; margin-right: 8px">
        <div class="card-image">
          <img alt="Header-Image, showing the building" src="/cdn/header/mi_0.webp" class="img-responsive" />
        </div>
        <div class="card-header">
          <div class="card-title h6" style="text-overflow: ellipsis; overflow: hidden; white-space: nowrap">
            Teilbibliothek Stammgelände
          </div>
          <small class="card-subtitle text-gray">Teilbibliothek</small>-->
          <div class="divider"></div>
        </div>
        <div class="card-body"></div>
        <div class="card-footer">
          <button class="btn btn-primary">Mehr Infos</button>
        </div>
      </div>
      <div
        class="card"
        style="width: 250px; display: inline-flex; margin-right: 8px; height: 200px; vertical-align: top"
      >
        <div class="card-image" style="display: none">
          <img alt="Header-Image, showing the building" src="/cdn/header/mi_0.webp" class="img-responsive" />
        </div>
        <div class="card-header">
          <div class="card-title h6" style="text-overflow: ellipsis; overflow: hidden; white-space: nowrap">
            Validierungsautomaten
          </div>
          <small class="card-subtitle text-gray" style="display: none">Teilbibliothek</small>
        </div>
        <div class="card-body" style="overflow-y: auto; /*! position: relative; */ /*! overflow-x: hidden; */">
          <div class="tile tile-centered">
            <div class="tile-icon">
              <div class="example-tile-icon">
                <i class="icon icon-location centered"></i>
              </div>
            </div>
            <div class="tile-content">
              <div class="tile-title">
                <a href="#/view/mi" class="btn btn-link">Süd-Seite</a>
              </div>
            </div>
            <div class="tile-action">
              <button class="btn btn-link">
                <i class="icon icon-arrow-right"></i>
              </button>
            </div>
          </div>
        </div>
      </div>
      <div
        style="
          position: absolute;
          height: 100%;
          display: flex;
          background: linear-gradient(90deg, #fff0 0%, #fffd 100%);
          right: 0;
          top: 0;
        "
      >
        <button class="btn btn-action s-circle" style="margin: auto 0; /*! position: absolute; */">
          <i class="icon icon-arrow-right"></i>
        </button>
      </div>
    </div>
  </div>
</template>

<script lang="ts"></script>
