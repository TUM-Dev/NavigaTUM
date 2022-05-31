function searchNavigateTo(to, from, next, component) {
    navigatum.beforeNavigate(to, from);

    const params = new URLSearchParams();
    params.append('q', to.query.q);
    params.append('limit_buildings', '10');
    params.append('limit_rooms', '30');
    params.append('limit_all', '30');

    cached_fetch
        .fetch(`${navigatum.api_base}search?${params.toString()}`, { cache: 'no-cache' })
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

const _search_default_state = {};

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
        };
    },
    beforeRouteEnter: function(to, from, next) {
        searchNavigateTo(to, from, next, null);
    },
    beforeRouteUpdate: function(to, from, next) {
        searchNavigateTo(to, from, next, this);
    },
    methods: {
        genDescription: function(data) {
            let sections_descr = '';
            let found_total = 0;
            for (const section of data.sections) {
                if (section.estimatedTotalHits) {
                    let facet_str;
                    if (section.facet === 'sites_buildings') {
                        facet_str = '${{ _.search.sections.buildings }}$';
                        if (section.estimatedTotalHits !== section.n_visible) {
                            const visible_str = '${{ _.search.sections.of_which_visible }}$';
                            facet_str = `(${section.n_visible} ${visible_str}) ${facet_str}`;
                        }
                    } else facet_str = '${{ _.search.sections.rooms }}$';
                    if (found_total > 0) sections_descr += ' ${{ _.search.sections.and }}$ ';
                    sections_descr += `${section.estimatedTotalHits} ${facet_str}`;
                }
                found_total += section.estimatedTotalHits;
            }
            if (found_total === 0)
                sections_descr = '${{ _.search.sections.no_buildings_rooms_found }}$';
            else sections_descr += ' ${{ _.search.sections.were_found }}$';
            return sections_descr;
        },
        loadSearchData: function(query, data) {
            this.search_data = data;
            this.query = query;
            navigatum.app.search.query = query;
            navigatum.setTitle(`\${{ _.view_search.search_for }}$ "${query}"`);
            navigatum.setDescription(this.genDescription(data));
            // Currently borrowing this functionality from autocomplete.
            // In the future it is planned that this search results page
            // has a different format.
            const _this = this;
            navigatum.getModule('autocomplete').then(function (c) {
                _this.sections = c.extract_facets(data);
            });
        },
    },
});
