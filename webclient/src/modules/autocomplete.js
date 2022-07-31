navigatum.registerModule(
  "autocomplete",
  (() => {
    function getVisibleElements() {
      const visible = [];

      navigatum.app.search.autocomplete.sections.forEach((section) => {
        section.entries.forEach((entry, index) => {
          if (
            section.n_visible === undefined ||
            index < section.n_visible ||
            section.expanded
          )
            visible.push(entry.id);
        });
      });
      return visible;
    }

    function extractFacets(data) {
      const sections = [];

      data.sections.forEach((section) => {
        const entries = [];

        section.entries.forEach((entry) => {
          // Search uses DC3 and DC1 to mark the beginning/end
          // of a highlighted sequence:
          // https://en.wikipedia.org/wiki/C0_and_C1_control_codes#Modified_C0_control_code_sets
          const name = new Option(entry.name).innerHTML
            .replaceAll("\x19", "<em>")
            .replaceAll("\x17", "</em>");
          const parsedId = new Option(entry.parsed_id).innerHTML
            .replaceAll("\x19", "<em>")
            .replaceAll("\x17", "</em>");
          const subtextBold = new Option(entry.subtext_bold).innerHTML
            .replaceAll("\x19", "<em>")
            .replaceAll("\x17", "</em>");
          entries.push({
            id: entry.id,
            name: name,
            type: entry.type,
            subtext: entry.subtext,
            subtext_bold: subtextBold,
            parsed_id: parsedId,
          });
        });

        if (section.facet === "sites_buildings") {
          sections.push({
            name: "${{ _.search.sections.buildings }}$",
            expanded: false,
            entries: entries,
            estimatedTotalHits: section.estimatedTotalHits,
            n_visible: section.n_visible,
          });
        } else if (section.facet === "rooms") {
          sections.push({
            name: "${{ _.search.sections.rooms }}$",
            entries: entries,
            estimatedTotalHits: section.estimatedTotalHits,
          });
        }
      });

      return sections;
    }

    // As a simple measure against out-of-order responses
    // to the autocompletion, we count queries and make sure
    // that late results will not overwrite the currently
    // visible results.
    let queryCounter = 0;
    let latestUsedQueryId = null;

    return {
      init: function () {},
      onInput: function (q) {
        navigatum.app.search.autocomplete.highlighted = null;

        if (q.length === 0) {
          navigatum.app.search.autocomplete.sections = [];
        } else {
          const queryId = queryCounter;
          queryCounter += 1;

          /* global cachedFetch */
          // no-cache instructs browser, because the cachedFetch will store the reponse.
          const cacheConfig = { cache: "no-cache" };
          cachedFetch
            .fetch(
              `${navigatum.apiBase}search?q=${window.encodeURIComponent(q)}`,
              cacheConfig
            )
            .then((data) => {
              // Data will be cached anyway in case the user hits backspace,
              // but we need to discard the data here if it arrived out of order.
              if (!latestUsedQueryId || queryId > latestUsedQueryId) {
                latestUsedQueryId = queryId;

                navigatum.app.search.autocomplete.sections =
                  extractFacets(data);
              }
            });
        }
      },
      extractFacets: extractFacets,
      onKeyDown: function (e) {
        let visible;
        let index;
        switch (e.keyCode) {
          case 27: // ESC
            document.getElementById("search").blur();
            break;

          case 40: // Arrow down
            visible = getVisibleElements();
            index = visible.indexOf(
              navigatum.app.search.autocomplete.highlighted
            );
            if (index === -1 && visible.length > 0) {
              navigatum.app.search.autocomplete.highlighted = visible[0];
            } else if (index >= 0 && index < visible.length - 1) {
              navigatum.app.search.autocomplete.highlighted =
                visible[index + 1];
            }
            e.preventDefault();
            break;

          case 38: // Arrow up
            visible = getVisibleElements();
            index = visible.indexOf(
              navigatum.app.search.autocomplete.highlighted
            );
            if (index === 0) {
              navigatum.app.search.autocomplete.highlighted = null;
            } else if (index > 0) {
              navigatum.app.search.autocomplete.highlighted =
                visible[index - 1];
            }
            e.preventDefault();
            break;

          case 13: // Enter
            if (navigatum.app.search.autocomplete.highlighted !== null) {
              navigatum.app.searchGoTo(
                navigatum.app.search.autocomplete.highlighted,
                true
              );
            } else {
              navigatum.app.searchGo(false);
            }
            break;
          default:
            break;
        }
      },
    };
  })()
);
