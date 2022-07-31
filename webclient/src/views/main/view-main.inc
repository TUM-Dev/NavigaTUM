<div id="view-main" v-if="root_data">
  <div class="columns" style="margin-top: 25px">
    <div class="column"><h5>${{_.view_main.sites}}$</h5></div>
    <!--<div class="column col-auto"><a href="#"><i class="icon icon-location"></i> ${{_.view_main.overview_map}}$</a></div>-->
  </div>
  <div class="columns">
    <div
      class="column col-6 col-xs-12"
      v-for="site in root_data.sites_overview"
    >
      <div class="panel" v-bind="{'id': 'panel-' + site.id}">
        <div class="panel-header">
          <router-link v-bind:to="'/view/' + site.id" v-if="site.id">
            <div class="columns">
              <div class="column">
                <div class="panel-title h6">{{ site.name }}</div>
              </div>
              <div class="column col-auto">
                <button
                  class="btn btn-link"
                  v-bind:style="{visibility: site.id ? '' : 'hidden'}"
                >
                  <i class="icon icon-forward"></i>
                </button>
              </div>
            </div>
          </router-link>
          <div class="columns" v-else>
            <div class="column">
              <div class="panel-title h6">{{ site.name }}</div>
            </div>
          </div>
        </div>
        <div class="panel-body">
          <router-link
            v-bind:to="'/view/' + c.id"
            v-for="(c, i) in site.children"
            v-bind:class="{'link-more': i >= site.n_visible}"
            v-bind:key="c.id"
            v-bind:aria-label="`show the details for the building '` + c.name + `'`"
          >
            <div class="tile tile-centered">
              <div class="tile-icon">
                <div class="example-tile-icon">
                  <i class="icon icon-location centered"></i>
                </div>
              </div>
              <div class="tile-content">
                <div class="tile-title">{{ c.name }}</div>
              </div>
              <div class="tile-action">
                <button
                  class="btn btn-link"
                  v-bind:aria-label="`show the details for the building '` + c.name + `'`"
                >
                  <i class="icon icon-arrow-right"></i>
                </button>
              </div>
            </div>
          </router-link>
          <button
            class="btn btn-link btn-more"
            aria-label="show more buildings"
            v-on:click="more(site.id)"
            v-if="site.children.length > site.n_visible"
          >
            <i class="icon icon-arrow-right"></i>
            ${{_.view_main.more}}$
          </button>
          <button
            class="btn btn-link btn-less"
            aria-label="show less buildings"
            v-on:click="less(site.id)"
          >
            <i class="icon icon-arrow-up"></i>
            ${{_.view_main.less}}$
          </button>
        </div>
      </div>
    </div>
  </div>
</div>
