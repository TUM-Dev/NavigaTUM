<div id="view-search" v-if="search_data">

  <h1>${{ _.view_search.search_for }}$ <i>"{{ query }}"</i></h1>
  
  <small class="search_meta">
    ${{ _.view_search.runtime }}$: {{ search_data.time_ms }}ms –
    <a onclick="open_feedback('search')" class="c-hand">${{_.view_search.give_feedback}}$</a>
  </small>

  <div class="divider"></div>
  
  <template v-for="s in sections">
    <section>
      <div class="columns">
        <div class="column">
          <h2>{{ s.name }}</h2>
        </div>
      </div>
      <ul class="result-list">
        <li v-for="e in s.entries">
          <router-link v-bind:to="'/view/' + e.id" class="tile tile-centered">
            <div class="tile-icon">
              <template v-if="e.type == 'room' || e.type == 'virtual_room'">
                <i v-if="e.parsed_id" class="icon icon-search"></i>
                <i v-else class="icon icon-location"></i>
              </template>
              <img v-else
                  class="avatar avatar-sm"
                  src="<!-- @echo app_prefix -->assets/thumb-building.png">
            </div>
            <div class="tile-content">
              <div class="tile-title">
                <span v-if="e.parsed_id" v-html="e.parsed_id"></span>
                          <i v-if="e.parsed_id" class="icon icon-caret"></i>
                          <span v-html="e.name"></span>
              </div>
              <small class="tile-subtitle text-gray">
                {{ e.subtext }}<template v-if="e.subtext_bold">, <b v-html="e.subtext_bold"></b></template>
              </small>
            </div>
            <!--<div class="tile-action">
              <button class="btn btn-link">
                <i class="icon icon-more-vert"></i>
              </button>
            </div>-->
          </router-link>
        </li>
      </ul>
      <div class="search-comment nb_results">
        {{ s.nb_hits > 20 ? "${{_.search.approx}}$ " : "" }}{{ s.nb_hits }}
        {{ s.nb_hits === 1 ? "${{_.search.result}}$" : "${{_.search.results}}$" }}{{ s.nb_hits > 10 ? ", ${{_.view_search.max_results }}$" : ""}}
      </div>
    </section>
  </template>
</div>
