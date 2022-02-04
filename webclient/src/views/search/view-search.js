function searchNavigateTo(to, from, next, component) {
    navigatum.beforeNavigate(to, from);

    cached_fetch.fetch(navigatum.api_base + 'search/' + window.encodeURI(to.params.query) +
                       '?limit_buildings=10&limit_rooms=20&limit_all=30',
                                   {cache: "no-cache"})
        .then(resp => {
            if (component) {
                next();
                navigatum.afterNavigate(to, from);
                component.loadSearchData(to.params.query, resp);
            } else {
                next(vm => {
                    navigatum.afterNavigate(to, from);
                    vm.loadSearchData(to.params.query, resp);
                });
            }
        });
}

var _search_default_state = {};

navigatum.registerView('search', {
    name: 'view-search',
    template: { gulp_inject: 'view-search.inc' },
    data: function() {
        return {
            search_data: null,
            sections: null,
            query: null,
            // State is preserved when navigating in history.
            // May only contain serializable objects!
            state: navigatum.cloneState(_search_default_state),
        }
    },
    beforeRouteEnter: function(to, from, next) { searchNavigateTo(to, from, next, null) },
    beforeRouteUpdate: function(to, from, next) { searchNavigateTo(to, from, next, this) },
    methods: {
        loadSearchData: function(query, data) {
            this.search_data = data;
            this.query = query;
            var search= document.getElementById("search")
            if (search.value.length === 0) {
                // /search/:querry is called from an internal url and thus the search url is not set
                search.value = query;
            }
            navigatum.setTitle('${{ _.view_search.search_for }}$ "' + query + '"');
            
            // Currently borrowing this functionality from autocomplete.
            // In the future it is planned that this search results page
            // has a different format.
            var _this = this;
            navigatum.getModule("autocomplete").then(function(c) {
                var sections = c.extract_facets(data);
                _this.sections = sections;
            });
        },
    }
})
