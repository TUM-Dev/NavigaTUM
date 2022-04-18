<div id="view-view" v-if="view_data">
  <!-- Header image (on mobile) -->
  <div class="show-sm header-image-mobile">
    <img alt="Header-Image, showing the building"
         v-bind:src="'<!-- @echo cdn_prefix -->header/' + view_data.img[0].name"
         v-if="view_data.img && view_data.img[0]"
         class="img-responsive"/>
  </div>

  <!-- Breadcrumbs -->
  <ol class="breadcrumb"
      vocab="https://schema.org/" typeof="BreadcrumbList">
    <li class="breadcrumb-item"
        v-for="(p, i) in view_data.parent_names"
        property="itemListElement" typeof="ListItem">
      <router-link v-bind="{'to': '/view/' + view_data.parents[i]}"
                   property="item" typeof="WebPage">
        {{ p }}
      </router-link>
      <meta property="position" v-bind:content="i+1">
    </li>
  </ol>

  <!-- Entry header / title -->
  <div class="entry-header">
    <div class="title">
      <div class="hide-sm">
        <button class="btn btn-link btn-action btn-sm"
                title="${{ _.view_view.header.copy_link }}$"
                @click="copy_link">
          <i class="icon icon-check" v-if="copied"></i>
          <i class="icon icon-link" v-else></i>
        </button>
      </div>
      <h1>{{ view_data.name }}<!-- <small class="label">Exaktes Ergebnis</small>--></h1>
    </div>
    <div class="columns subtitle">
      <div class="column col-auto">
        <span>{{ view_data.type_common_name }}</span>
      </div>
      <div class="column col-auto col-ml-auto">
        <button class="btn btn-link btn-action btn-sm"
                title="${{ _.view_view.header.feedback }}$"
                @click="entry_feedback(view_data.id)">
          <i class="icon icon-flag"></i>
        </button>
        <!--<button class="btn btn-link btn-action btn-sm"
                title="${{_.view_view.header.favorites}}$">
          <i class="icon icon-bookmark"></i>
        </button>-->
      </div>
    </div>
    <div class="divider"></div>
  </div>

  <!-- First info section (map + infocard) -->
  <div class="columns">
    <!-- Map container -->
    <div class="column col-7 col-md-12" id="map-container">
      <div class="show-sm">
        <div class="toast toast-warning"
             v-if="view_data.coords.accuracy && view_data.coords.accuracy === 'building'">
          ${{_.view_view.msg.inaccurate_only_building}}$
        </div>
      </div>

      <div id="interactive-map-container"
           v-bind:class="{'d-none': state.map.selected !== 'interactive'}">
        <div>
          <div id="interactive-map" class="loading"></div>
        </div>
      </div>
      <div class="roomfinder-map-container"
           v-bind:class="{'d-none': state.map.selected !== 'roomfinder'}"
           v-if="view_data.maps.roomfinder && view_data.maps.roomfinder.available">
        <img alt="Cross showing where the room is located on the hand-drawn roomfinder map image"
             src="<!-- @echo app_prefix -->assets/roomfinder_cross-v2.webp"
             v-bind:style="{'transform': 'translate(' + state.map.roomfinder.x + 'px, ' + state.map.roomfinder.y + 'px)'}"
             id="roomfinder-map-cross">
        <img alt="Hand-drawn roomfinder map image"
             v-bind:src="'<!-- @echo cdn_prefix -->maps/roomfinder/' + view_data.maps.roomfinder.available[state.map.roomfinder.selected_index].path"
             class="img-responsive"
             v-bind:width="state.map.roomfinder.width"
             v-bind:height="state.map.roomfinder.height"
             id="roomfinder-map-img">
        <div>${{_.view_view.map.img_source}}$: {{ view_data.maps.roomfinder.available[state.map.roomfinder.selected_index].source }}</div>
      </div>
      <div class="accordion"
           id="roomfinder-map-select"
           v-bind:class="{'d-none': state.map.selected !== 'roomfinder'}"
           v-if="view_data.maps.roomfinder && view_data.maps.roomfinder.available">
        <input id="map-accordion" type="checkbox" name="accordion-checkbox" hidden>
        <label for="map-accordion" class="btn btn-sm btn-block accordion-header">
          1:{{ view_data.maps.roomfinder.available[state.map.roomfinder.selected_index].scale }},
          {{ view_data.maps.roomfinder.available[state.map.roomfinder.selected_index].name }}
          <i class="icon icon-caret"></i>
        </label>
        <div class="accordion-body" v-if="view_data.maps && view_data.maps.roomfinder">
          <ul class="menu menu-nav">
            <li class="menu-item"
                v-for="(m, i) in view_data.maps.roomfinder.available">
                <button class="btn btn-sm"
                        v-bind:aria-label="`show the map '` + m.name + `' at the scale 1:` + m.scale"
                        v-bind:class="{selected: m.id==state.map.roomfinder.selected_id}"
                        v-on:click="loadRoomfinderMap(i, true)">
                1:{{ m.scale }}, {{ m.name }}
              </button>
            </li>
          </ul>
        </div>
      </div>
      <div class="btn-group btn-group-block">
        <button class="btn btn-sm"
                v-on:click="loadInteractiveMap(true)"
                v-bind:class="{active: state.map.selected === 'interactive'}">
          ${{_.view_view.map.interactive}}$
        </button>
        <button class="btn btn-sm"
                v-on:click="loadRoomfinderMap(state.map.roomfinder.selected_index, true)"
                v-bind:class="{active: state.map.selected === 'roomfinder'}"
                v-bind:disabled="!(view_data.maps.roomfinder && view_data.maps.roomfinder.available)">
          ${{_.view_view.map.roomfinder}}$
        </button>
      </div>
      <div class="divider" style="margin-top: 10px;"></div>
    </div>

    <!-- Information section (on mobile) -->
    <div class="column col-5 col-sm-12 show-sm mobile-info-section"
         v-if="view_data.props && view_data.props.computed">
      <h2>Informationen</h2>
      <table class="info-table">
        <tbody>
          <tr v-for="prop in view_data.props.computed">
            <td><strong>{{ prop.name }}</strong></td>
            <td>{{ prop.text }}</td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Informationen card (desktop) -->
    <!-- Some elements are currently duplicate, which is not optimal but should be okay
         as long as only little information is there -->
    <div class="column col-5 col-md-12 hide-sm">
      <div class="card">
        <div class="card-image">
          <img alt="Header-Image, showing the building"
               v-bind:src="'<!-- @echo cdn_prefix -->header/' + view_data.img[0].name"
               v-if="view_data.img && view_data.img[0]"
               class="img-responsive"
               width="100%"/>
        </div>
        <div class="card-header">
          <div class="card-title h5">${{_.view_view.info_title}}$</div>
        </div>
        <div class="card-body">
          <table class="info-table"
                 v-if="view_data.props && view_data.props.computed">
            <tbody>
              <tr v-for="prop in view_data.props.computed">
                <td><strong>{{ prop.name }}</strong></td>
                <td>{{ prop.text }}</td>
              </tr>
            </tbody>
          </table>
          <span v-else>-</span>
          <div class="toast toast-warning"
               v-if="view_data.coords.accuracy && view_data.coords.accuracy === 'building'">
            ${{_.view_view.msg.inaccurate_only_building}}$
          </div>
        </div>
        <!--<div class="card-footer">
            <button class="btn btn-link">Mehr Infos</button>
        </div>-->
      </div>
    </div>
  </div>

<!-- TMP -->
<!--
<div v-if="view_data.sections && view_data.sections.featured">
<div class="columns" style="margin-top: 40px">
    <div class="column"><h2>Featured</h2></div>
</div>
<div style="position: relative;overflow: hidden;white-space: nowrap;">
    <div style="position: absolute;height: 100%;display: flex;background: linear-gradient(-90deg, #fff0 0%, #fffd 100%);">
        <button class="btn btn-action s-circle" style="margin: auto 0;/*! position: absolute; */"><i class="icon icon-arrow-left"></i></button>
    </div>
    <div class="card" style="width: 250px;display: inline-flex;margin-right: 8px;">
        <div class="card-image">
            <img alt="Header-Image, showing the building"
                 src="<!-- @echo cdn_prefix -- >header/mi_0.webp"
                 class="img-responsive"/>
        </div>
        <div class="card-header">
            <div class="card-title h6" style="text-overflow: ellipsis;overflow: hidden;white-space: nowrap;">Teilbibliothek Stammgelände</div>
            <small class="card-subtitle text-gray">Teilbibliothek</small>
            <!--<div class="divider"></div>-- >
        </div>
        <div class="card-body">
            
        </div>
        <!--<div class="card-footer">
            <button class="btn btn-primary">Mehr Infos</button>
        </div>-- >
    </div>
    <div class="card" style="width: 250px;display: inline-flex;margin-right: 8px;height: 200px;vertical-align: top;">
        <div class="card-image" style="display: none;">
            <img alt="Header-Image, showing the building"
                 src="<!-- @echo cdn_prefix -- >header/mi_0.webp"
                 class="img-responsive">
        </div>
        <div class="card-header">
            <div class="card-title h6" style="text-overflow: ellipsis; overflow: hidden; white-space: nowrap;">Validierungsautomaten</div>
            <small class="card-subtitle text-gray" style="display: none;">Teilbibliothek</small>
        </div>
        <div class="card-body" style="overflow-y: auto;/*! position: relative; *//*! overflow-x: hidden; */">
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
                    <button class="btn btn-link"><i class="icon icon-arrow-right"></i></button>
                </div>
            </div>
        </div>
    </div>
    <div style="position: absolute;height: 100%;display: flex;background: linear-gradient(90deg, #fff0 0%, #fffd 100%);right: 0;top: 0;">
        <button class="btn btn-action s-circle" style="margin: auto 0;/*! position: absolute; */"><i class="icon icon-arrow-right"></i></button>
    </div>
</div>
</div>-->

  <!-- Buildings overview -->
  <section v-if="view_data.sections && view_data.sections.buildings_overview"
           id="building-overview">
    <div class="columns">
      <div class="column"><h2>${{_.view_view.buildings_overview.title}}$</h2></div>
      <!--<div class="column col-auto">
        <a href="#">Übersichtskarte <i class="icon icon-forward"></i></a>
      </div>-->
    </div>
    <div class="columns">
      <div class="column col-4 col-md-12 content"
           v-for="(b, i) in view_data.sections.buildings_overview.entries"
           v-if="i < view_data.sections.buildings_overview.n_visible || state.buildings_overview.expanded">
        <router-link v-bind:to="'/view/' + b.id">
          <div class="tile tile-centered">
            <div class="tile-icon">
              <figure class="avatar avatar-lg">
                <img v-bind:alt="b.thumb ? 'Thumbnail, showing a preview of the building.' : 'Default-thumbnail, as no thumbnail is available'"
                     v-bind:src="b.thumb ? '<!-- @echo cdn_prefix -->thumb/' + b.thumb : '<!-- @echo app_prefix -->assets/thumb-building.webp'">
              </figure>
            </div>
            <div class="tile-content">
              <p class="tile-title">{{ b.name }}</p>
              <small class="tile-subtitle text-dark">{{ b.subtext }}</small>
            </div>
            <div class="tile-action">
              <button class="btn btn-link"
                      v-bind:aria-label="`show the details for the building '`+ b.name +`'`">
                <i class="icon icon-arrow-right"></i>
              </button>
            </div>
          </div>
        </router-link>
      </div>
    </div>
    <div v-if="view_data.sections.buildings_overview.n_visible < view_data.sections.buildings_overview.entries.length">
      <button class="btn btn-link"
              v-if="!state.buildings_overview.expanded"
              v-on:click="state.buildings_overview.expanded = true">
        <i class="icon icon-arrow-right"></i> ${{_.view_view.buildings_overview.more}}$
      </button>
      <button class="btn btn-link"
              v-if="state.buildings_overview.expanded"
              v-on:click="state.buildings_overview.expanded = false">
        <i class="icon icon-arrow-up"></i> ${{_.view_view.buildings_overview.less}}$
      </button>
    </div>
  </section>

  <!-- Rooms overview -->
  <section id="rooms-overview"
           v-if="view_data.sections && view_data.sections.rooms_overview">
    <div class="columns">
      <div class="column"><h2>${{_.view_view.rooms_overview.title}}$</h2></div>
      <!--<div class="column col-auto">
        <div class="dropdown"><a class="btn btn-link dropdown-toggle" tabindex="0">${{_.view_view.rooms_overview.by_usage}}$ <i class="icon icon-caret"></i></a>
          <ul class="menu">
                  <li class="menu-item"><a href="#dropdowns">nach Nutzung</a></li>
                  <li class="menu-item"><a href="#dropdowns">nach ...</a></li>
          </ul>
        </div>
      </div>-->
    </div>

    <div class="columns content">
      <div class="column col-4 col-lg-5 col-md-6 col-sm-12" id="rooms-overview-select">
        <div class="panel">
          <div class="panel-header">
            <div class="panel-title h6">
              ${{_.view_view.rooms_overview.by_usage}}$:
            </div>
          </div>
          <div class="panel-body">
            <ul class="menu">
              <li class="menu-item">
                <button class="btn"
                        v-bind:class="{active: state.rooms_overview.selected === -1}"
                   v-on:click="updateRoomsOverview(-1)">
                  <i class="icon icon-arrow-right"></i>
                  <div class="menu-text">
                    ${{_.view_view.rooms_overview.any}}$
                  </div>
                  <label class="label">{{ sections.rooms_overview.combined_count }}</label>
                </button>
              </li>
              <li class="divider" data-content=""></li>
              <li class="menu-item"
                  v-for="(u, i) in view_data.sections.rooms_overview.usages">
                <button class="btn"
                        v-bind:class="{active: i == state.rooms_overview.selected}"
                   v-on:click="updateRoomsOverview(i)">
                  <i class="icon icon-arrow-right"></i>
                  <div class="menu-text">
                    {{ u.name }}
                  </div>
                  <label class="label">{{ u.count }}</label>
                </button>
              </li>
            </ul>
          </div>
          <div class="panel-footer">
            <button class="btn btn-link btn-sm"
                    v-on:click="updateRoomsOverview(null)">
              ${{_.view_view.rooms_overview.remove_selection}}$
            </button>
          </div>
        </div>
      </div>
      <div class="column col-8 col-lg-7 col-md-6 col-sm-12 hide-l" id="rooms-overview-list">
        <div class="show-sm" style="height: 15px"></div>
        <div class="panel">
          <div class="panel-header">
            <div class="input-group">
              <input v-model="state.rooms_overview.filter"
                     placeholder="${{_.view_view.rooms_overview.filter}}$"
                     class="form-input">
              <button class="btn btn-primary input-group-btn"
                      @click="state.rooms_overview.filter = ''"
                      aria-label="Clear the filter">
                <i class="icon icon-cross"></i>
              </button>
            </div>
          </div>
          <div class="panel-body">
            <div v-bind:class="{loading: sections.rooms_overview.loading}"></div>
            <ul class="menu"
                v-if="state.rooms_overview.selected !== null">
                <li class="menu-item"
                    v-for="r in sections.rooms_overview.display_list">
                    <router-link v-bind:to="'/view/' + r.id">
                        <i class="icon icon-location"></i> {{ r.name }}
                    </router-link>
                </li>
            </ul>
          </div>
          <div class="panel-footer">
            <small>
              {{ state.rooms_overview.selected === null
                 ? "${{_.view_view.rooms_overview.choose_usage}}$"
                 : sections.rooms_overview.display_list.length
                   + " ${{_.view_view.rooms_overview.result}}$"
                   + (sections.rooms_overview.display_list.length === 1
                     ? "" : "${{_.view_view.rooms_overview.results_suffix}}$")
                   + (state.rooms_overview.filter === ""
                     ? "" : " (${{_.view_view.rooms_overview.filtered}}$)") }}
            </small>
          </div>
        </div>
      </div>
    </div>
  </section>

  <section id="entry-sources">
    <div class="columns">
        <div class="column"><h2>${{_.view_view.sources.title}}$</h2></div>
    </div>
    <p v-if="">
      ${{_.view_view.sources.base.title}}$:
      <span v-for="(e, i) in view_data.sources.base">
        <a v-if="e.url" v-bind:href="e.url">{{e.name}}</a>
        <template v-else>{{e.name}}</template>
        <template v-if="i<view_data.sources.base.length-1">&#32;•&#32;</template>
      </span>
      <span v-if="view_data.sources.patched"><br>${{_.view_view.sources.base.patched}}$</span>
    </p>
    <p v-if="(view_data.img && view_data.img[0])">
      ${{_.view_view.sources.header_img}}$:
      <span>{{ view_data.img[0].author.text }}</span>
      <span v-if="view_data.img[0].source">•
        <a v-if="view_data.img[0].source.url"
           v-bind:href="view_data.img[0].source.url"
           target="_blank">
          {{ view_data.img[0].source.text }}
        </a>
        <template v-else>{{ view_data.img[0].source.text }}</template>
      </span>
      <span v-if="view_data.img[0].license">&#32;•
        <a v-if="view_data.img[0].license.url"
           v-bind:href="view_data.img[0].license.url"
           target="_blank">
          {{ view_data.img[0].license.text }}
        </a>
        <template v-else>{{ view_data.img[0].license.text }}</template>
      </span>
    </p>
    <p v-if="view_data.coords">
      ${{_.view_view.sources.coords.title}}$:
      <span v-if="view_data.coords.source=='navigatum'">${{_.view_view.sources.coords.navigatum}}$</span>
      <span v-if="view_data.coords.source=='roomfinder'">${{_.view_view.sources.coords.roomfinder}}$</span>
      <span v-if="view_data.coords.source=='inferred'">${{_.view_view.sources.coords.inferred}}$</span>
    </p>
  </section>

</div>
