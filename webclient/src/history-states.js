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
    const evt = document.createEvent( 'CustomEvent' );
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
  window.history.states = [];
  if (
    window.history.state &&
    window.history.state[stateIndexSymbol] !== undefined
  ) {
    window.history.stateIndex = window.history.state[stateIndexSymbol];
    for (let i = 0; i < window.history.stateIndex; i++) {
      const state = {};
      state[stateIndexSymbol] = i;
      window.history.states.push([state, "", null]);
    }
    window.history.states.push([window.history.state, "", null]);
  } else {
    window.history.stateIndex = 0;

    if (window.history.state) {
      window.history.states.push([window.history.state, "", null]);
    } else {
      const state = {};
      state[stateIndexSymbol] = 0;
      window.history.states.push([state, "", null]);
    }
  }

  window.history.lastStateIndex = null;
  const historyPushState = window.history.pushState;
  function add(data, title, url) {
    if (data == null) data = {};
    if (typeof data !== "object") data = { data: data };
    data[stateIndexSymbol] = window.history.stateIndex + 1;
    window.history.states.splice(window.history.stateIndex + 1, 0, [
      data,
      title,
      url,
    ]);
    window.history.states.splice(window.history.stateIndex + 2);
    window.history.stateIndex += 1;
  }
  window.history.saveCurrentViewState = function () {
    if (
      navigatum.router &&
      navigatum.router.currentRoute.matched[0] &&
      navigatum.router.currentRoute.matched[0].instances.default.state
    ) {
      const stateIndex =
        window.history.lastStateIndex === null
          ? window.history.stateIndex
          : window.history.lastStateIndex;

      window.history.states[stateIndex][0][stateDataSymbol] =
        navigatum.cloneState(
          navigatum.router.currentRoute.matched[0].instances.default.state
        );
    }
  };
  window.history.pushState = function (data, title, url = null) {
    add(data, title, url);
    historyPushState.bind(window.history)(data, title, url);
  };
  addEventListener("popstate", function (e) {
    // If navigation is window.history navigation (click on back/forward),
    // the 'popstate' event is emitted before 'beforeResolve()'.
    // So in this case, we need to temporarily store the old state index,
    // so that the saveCurrentViewState() call in beforeResolve() saves the
    // state to the correct state in the window.history.
    window.history.lastStateIndex = window.history.stateIndex;

    const eventObject = {};
    const newStateIndex =
      e.state != null && e.state[stateIndexSymbol] !== undefined
        ? e.state[stateIndexSymbol]
        : 0;
    eventObject.from = window.history.states[window.history.stateIndex];
    eventObject.to =
      newStateIndex in window.history.states
        ? window.history.states[newStateIndex]
        : null;
    eventObject.side =
      window.history.stateIndex > newStateIndex ? "back" : "forward";
    // This happens if there is a state in the window.history, that we have lost.
    // (usually happens on forward navigation from another page)
    if (!(newStateIndex in window.history.states)) {
      add(window.history.state, "", window.location.href);
    }
    // window.dispatchEvent(new CustomEvent("historyChange", {detail: eventObject} ))
    window.history.stateIndex =
      e.state != null && e.state[stateIndexSymbol] !== undefined
        ? e.state[stateIndexSymbol]
        : 0; // -1;
  });
})();

/* addEventListener("historyChange",function(e){
    const from = e.detail.from; // [ data , title , url ]
    const to   = e.detail.to;   // [ data , title , url ]
    const side = e.detail.side; // "back" | "forward"
    console.log(e);
}) */
