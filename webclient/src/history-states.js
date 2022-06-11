/*
  Author: Hasan Delibaş
  Source: https://gist.github.com/HasanDelibas/12050fc59d675181ea973d21f882081a
  Under MIT License
  
  modified
*/

// TODO: only if initialized and fallback==false

// CustomEvent polyfill (for IE)
/* (function () {
  if ( typeof window.CustomEvent === "function" ) return false;
  function CustomEvent ( event, params ) {
    params = params || { bubbles: false, cancelable: false, detail: null };
    var evt = document.createEvent( 'CustomEvent' );
    evt.initCustomEvent( event, params.bubbles, params.cancelable, params.detail );
    return evt;
   }
  window.CustomEvent = CustomEvent;
})(); */

(function () {
  const stateIndexSymbol = "__state__index__";
  const stateDataSymbol = "viewState";

  // If the page is reloaded, the state is preserved by the browser, but we have lost
  // the state list. For this reason we need to create a dummy one for now.
  // When navigating to that page again, we may be able to get the old state back (TODO).
  history.states = [];
  if (history.state && history.state[stateIndexSymbol] !== undefined) {
    history.stateIndex = history.state[stateIndexSymbol];
    for (let i = 0; i < history.stateIndex; i++) {
      var state = {};
      state[stateIndexSymbol] = i;
      history.states.push([state, "", null]);
    }
    history.states.push([history.state, "", null]);
  } else {
    history.stateIndex = 0;

    if (history.state) {
      history.states.push([history.state, "", null]);
    } else {
      var state = {};
      state[stateIndexSymbol] = 0;
      history.states.push([state, "", null]);
    }
  }

  history.lastStateIndex = null;
  const historyPushState = history.pushState;
  function add(data, title, url) {
    if (data == null) data = {};
    if (typeof data !== "object") data = { data: data };
    data[stateIndexSymbol] = history.stateIndex + 1;
    history.states.splice(history.stateIndex + 1, 0, [data, title, url]);
    history.states.splice(history.stateIndex + 2);
    history.stateIndex += 1;
  }
  history.saveCurrentViewState = function () {
    if (
      navigatum.router &&
      navigatum.router.currentRoute.matched[0] &&
      navigatum.router.currentRoute.matched[0].instances.default.state
    ) {
      const stateIndex =
        history.lastStateIndex === null
          ? history.stateIndex
          : history.lastStateIndex;

      history.states[stateIndex][0][stateDataSymbol] = navigatum.cloneState(
        navigatum.router.currentRoute.matched[0].instances.default.state
      );
    }
  };
  history.pushState = function (data, title, url = null) {
    add(data, title, url);
    historyPushState.bind(history)(data, title, url);
  };
  addEventListener("popstate", function (e) {
    // If navigation is history navigation (click on back/forward),
    // the 'popstate' event is emitted before 'beforeResolve()'.
    // So in this case, we need to temporarily store the old state index,
    // so that the saveCurrentViewState() call in beforeResolve() saves the
    // state to the correct state in the history.
    history.lastStateIndex = history.stateIndex;

    const eventObject = {};
    const newStateIndex =
      e.state != null && e.state[stateIndexSymbol] !== undefined
        ? e.state[stateIndexSymbol]
        : 0;
    eventObject.from = history.states[history.stateIndex];
    eventObject.to =
      newStateIndex in history.states ? history.states[newStateIndex] : null;
    eventObject.side = history.stateIndex > newStateIndex ? "back" : "forward";
    // This happens if there is a state in the history, that we have lost.
    // (usually happens on forward navigation from another page)
    if (!(newStateIndex in history.states)) {
      add(history.state, "", window.location.href);
    }
    // window.dispatchEvent(new CustomEvent("historyChange", {detail: eventObject} ))
    history.stateIndex =
      e.state != null && e.state[stateIndexSymbol] !== undefined
        ? e.state[stateIndexSymbol]
        : 0; // -1;
  });
})();

/* addEventListener("historyChange",function(e){
    var from = e.detail.from; // [ data , title , url ]
    var to   = e.detail.to;   // [ data , title , url ]
    var side = e.detail.side; // "back" | "forward"
    console.log(e);
}) */
