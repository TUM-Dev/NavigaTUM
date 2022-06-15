function searchNavigateTo(to, from, next, component) {
  navigatum.beforeNavigate(to, from);

  const params = new URLSearchParams();
  params.append("q", to.query.q);
  params.append("limit_buildings", "10");
  params.append("limit_rooms", "30");
  params.append("limit_all", "30");

  /* global cachedFetch */
  cachedFetch
    .fetch(`${navigatum.apiBase}search?${params.toString()}`, {
      cache: "no-cache",
    })
    .then((resp) => {
      if (component) {
        next();
        navigatum.afterNavigate(to, from);
        component.loadSearchData(to.query.q, resp);
      } else {
        next((vm) => {
          navigatum.afterNavigate(to, from);
          vm.loadSearchData(to.query.q, resp);
        });
      }
    });
}

const _searchDefaultState = {};

navigatum.registerView("search", {
  name: "view-search",
  template: { gulp_inject: "view-search.inc" },
  data: function () {
    return {
      search_data: null,
      sections: null,
      query: null,
      // State is preserved when navigating in history.
      // May only contain serializable objects!
      state: navigatum.cloneState(_searchDefaultState),
    };
  },
  beforeRouteEnter: function (to, from, next) {
    searchNavigateTo(to, from, next, null);
  },
  beforeRouteUpdate: function (to, from, next) {
    searchNavigateTo(to, from, next, this);
  },
  methods: {
    genDescription: function (data) {
      let sectionsDescr = "";
      let estimatedTotalHits = 0;
      for (const section of data.sections) {
        if (section.estimatedTotalHits) {
          let facetStr;
          if (section.facet === "sites_buildings") {
            facetStr = "${{ _.search.sections.buildings }}$";
            if (section.estimatedTotalHits !== section.n_visible) {
              const visibleStr = "${{ _.search.sections.of_which_visible }}$";
              facetStr = `(${section.n_visible} ${visibleStr}) ${facetStr}`;
            }
          } else facetStr = "${{ _.search.sections.rooms }}$";
          if (estimatedTotalHits > 0)
            sectionsDescr += " ${{ _.search.sections.and }}$ ";
          sectionsDescr += `${section.estimatedTotalHits} ${facetStr}`;
        }
        estimatedTotalHits += section.estimatedTotalHits;
      }
      if (estimatedTotalHits === 0)
        sectionsDescr = "${{ _.search.sections.no_buildings_rooms_found }}$";
      else sectionsDescr += " ${{ _.search.sections.were_found }}$";
      return sectionsDescr;
    },
    loadSearchData: function (query, data) {
      this.search_data = data;
      this.query = query;
      navigatum.app.search.query = query;
      navigatum.setTitle(`\${{ _.view_search.search_for }}$ "${query}"`);
      navigatum.setDescription(this.genDescription(data));
      // Currently borrowing this functionality from autocomplete.
      // In the future it is planned that this search results page
      // has a different format.
      const _this = this;
      navigatum.getModule("autocomplete").then(function (c) {
        _this.sections = c.extractFacets(data);
      });
    },
  },
});
