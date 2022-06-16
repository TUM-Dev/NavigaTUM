/*
 * This is the first JS code executed for all views.
 */

let navigatum;

// This is a wrapper around fetch that avoids duplicate requests if the
// same resource is requested another time before the first request has
// returned.
const cachedFetch = (function () {
  return {
    fetch: function (url, options) {
      return new Promise((resolve) => {
        if (url in this.cache) {
          resolve(this.cache[url]);
        } else if (url in this.promise_callbacks) {
          this.promise_callbacks[url].push(resolve);
        } else {
          this.promise_callbacks[url] = [resolve];
          if (!options.headers) options.headers = {};
          fetch(url, options)
            .then((response) => {
              if (!response.ok) {
                if (response.status === 404)
                  throw new Error("${{_.core_js.error.404}}$");
                else if (response.status === 500)
                  throw new Error("${{_.core_js.error.500}}$");
                else if (response.status === 503)
                  throw new Error("${{_.core_js.error.503}}$");
                else {
                  const errorStatus = "${{_.core_js.error.status}}$";
                  throw new Error(`${errorStatus}$${response.status}`);
                }
              }
              navigatum.app.error.msg = null;
              return options.as_text ? response.text() : response.json();
            })
            .catch((error) => {
              let msg;
              if (error instanceof TypeError)
                msg = "${{_.core_js.error.network}}$";
              else msg = error.message;

              if (!msg) msg = "${{_.core_js.error.unknown}}$";

              console.warn("Error on fetch:", error);

              if (navigatum && navigatum.app) navigatum.app.error.msg = msg;

              return null;
            })
            .then((data) => {
              if (data !== null) this.cache[url] = data;

              this.promise_callbacks[url].forEach((callback) => {
                callback(data);
              });
              delete this.promise_callbacks[url];
            });
        }
      });
    },
    cache: {},
    promise_callbacks: {},
  };
})();

navigatum = (function () {
  const apiBase = "/* @echo api_prefix */";
  const cache = {};

  const views = {};
  const viewsResolveCallbacks = {};
  let routes;

  const router = null;
  const app = null; // This is the Vue.js app

  function _modulePostInit(_this, name, c) {
    _this.modules.initialized[name] = c;
    if (name in _this.modules.loaded) delete _this.modules.loaded[name];

    _this.module_promise_callbacks[name].forEach((callback) => {
      callback(c);
    });
    delete _this.module_promise_callbacks[name];
  }

  return {
    apiBase: apiBase,
    init: function () {
      // Init Vue.js
      this.router = new VueRouter({
        /* @if target="release" */
        mode: "history",
        base: "/* @echo app_prefix */",
        /* @endif */
        routes: this.routes,
        scrollBehavior: function (to, from, savedPosition) {
          if (savedPosition) {
            return savedPosition;
          }
          // Just returning (0, 0) does not work when the new page is
          // the same component and it got so small, that the current
          // position is now the bottom of the new page.
          // For this reason this extra call.
          document.getElementById("content").scrollIntoView();

          return { x: 0, y: 0, behavior: "smooth" };
        },
      });
      this.router.beforeEach((to, from, next) => {
        this.beforeNavigate(to, from);
        next();
      });
      this.router.afterEach((to, from) => {
        this.afterNavigate(to, from);
      });
      /* this.router.beforeResolve((to, from, next) => {
                next();
            }); */
      this.app = new Vue({
        router: this.router,
        el: "#app",
        data: {
          search: {
            focused: false,
            keep_focus: false,
            query: "",
            autocomplete: {
              sections: [],
              highlighted: null,
            },
          },
          error: {
            msg: null,
          },
        },
        methods: {
          searchFocus: function () {
            this.search.focused = true;
            this.search.autocomplete.highlighted = null;
          },
          searchBlur: function () {
            if (this.search.keep_focus) {
              window.setTimeout(function () {
                // This is relevant if the call is delayed and focused has
                // already been disabled e.g. when clicking on an entry.
                if (this.search.focused)
                  document.getElementById("search").focus();
              }, 0);
              this.search.keep_focus = false;
            } else {
              this.search.focused = false;
            }
          },
          searchInput: function (e) {
            navigatum.getModule("autocomplete").then(function (c) {
              c.onInput(e.srcElement.value);
            });
          },
          searchKeydown: function (e) {
            navigatum.getModule("autocomplete").then(function (c) {
              c.onKeyDown(e);
            });
          },
          searchExpand: function (s) {
            s.expanded = true;
          },
          searchGo: function (cleanQuery) {
            if (this.search.query.length === 0) return;

            navigatum.router
              .push(`/search?q=${this.search.query}`)
              .catch(() => {
                navigatum.afterNavigate();
              });
            this.search.focused = false;
            if (cleanQuery) {
              this.search.query = "";
              this.search.autocomplete.sections = [];
            }
            document.getElementById("search").blur();
          },
          searchGoTo: function (id, cleanQuery) {
            // Catch is necessary because vue-router throws an error
            // if navigation is aborted for some reason (e.g. the new
            // url is the same or there is a loop in redirects)
            navigatum.router.push(`/view/${id}`).catch(() => {
              navigatum.afterNavigate();
            });
            this.search.focused = false;
            if (cleanQuery) {
              this.search.query = "";
              this.search.autocomplete.sections = [];
            }
            document.getElementById("search").blur();
          },
          setLang: navigatum.setLang,
          setTheme: navigatum.setTheme,
        },
      });
    },
    cache: cache,
    /*
     * getData() either uses cachedFetch() to retrieve the specified data
     * or loads it from its local cache if present.
     */
    getData: function (id) {
      return new Promise((resolve) => {
        cachedFetch
          .fetch(`${this.apiBase}get/${window.encodeURIComponent(id)}`, {
            cache: "force-cache",
          })
          .then((data) => resolve(data));
      });
    },
    setLocalStorageWithExpiry: function (key, value, ttl) {
      // ttl in hours
      const now = new Date();

      const item = {
        value: value,
        expiry: now.getTime() + ttl * 3.6e6,
      };
      localStorage.setItem(key, JSON.stringify(item));

      // "storage" usually fires only across tabs, this way we
      // force it to fire in this window as well
      const e = new Event("storage");
      window.dispatchEvent(e);
    },
    getLocalStorageWithExpiry: function (key, defaultValue = null) {
      const itemStr = localStorage.getItem(key);
      if (!itemStr) {
        return defaultValue;
      }
      const item = JSON.parse(itemStr);
      const now = new Date();
      if (now.getTime() > item.expiry) {
        localStorage.removeItem(key);
        return defaultValue;
      }
      return item.value;
    },
    removeLocalStorage: function (key) {
      localStorage.removeItem(key);
      const e = new Event("storage");
      window.dispatchEvent(e);
    },
    /*
     * Views can be lazy loaded. Each view will call registerView() once it is
     * availabe. If the router requests a view with getView(), it can be directly
     * returned if it is already available. If not, it is retrieved and returned
     * as soon as it is availabe (viewsResolveCallbacks stores the callbacks for
     * this).
     */
    // NOTE: This code doesn't use `this` because for some reason it doesn't work with IE
    registerView: function (name, component) {
      if (!(name in navigatum.views)) navigatum.views[name] = component;
      if (name in viewsResolveCallbacks) {
        viewsResolveCallbacks[name](component);
        delete viewsResolveCallbacks[name];
      }
    },
    getView: function (name) {
      return function (resolve, reject) {
        if (name in navigatum.views) {
          resolve(navigatum.views[name]);
        } else {
          viewsResolveCallbacks[name] = resolve;
          window.setTimeout(function () {
            if (name in viewsResolveCallbacks) {
              if (navigatum.app)
                navigatum.app.error.msg =
                  "${{_.core_js.error.view_load_timeout}}$";
              reject("Load timed out");
            }
          }, 15000);
        }
      };
    },
    views: views,
    registerModule: function (name, c) {
      // If there are open promise callbacks for this module,
      // it initialized directly. Else it is only initialized when needed.
      if (name in this.module_promise_callbacks) {
        const res = c.init();
        if (!res) {
          // Init without Promise
          _modulePostInit(this, name, c);
        } else {
          // Init with Promise
          res.then(() => {
            _modulePostInit(this, name, c);
          });
        }
      } else {
        this.modules.loaded[name] = c;
      }
    },
    getModule: function (name, ...args) {
      return new Promise((resolve) => {
        if (name in this.modules.initialized) {
          resolve(this.modules.initialized[name]);
        } else {
          if (name in this.module_promise_callbacks) {
            this.module_promise_callbacks[name].push(resolve);
          } else {
            this.module_promise_callbacks[name] = [resolve];
          }

          // Init if already loaded
          if (name in this.modules.loaded) {
            const res = this.modules.loaded[name].init();
            if (!res) {
              // Init without Promise
              _modulePostInit(this, name, this.modules.loaded[name]);
            } else {
              // Init with Promise
              res.then((_) => {
                _modulePostInit(this, name, this.modules.loaded[name]);
              });
            }
          }
        }
      });
    },
    modules: {
      loaded: {
        /* removed here after init */
      },
      initialized: {},
    },
    module_promise_callbacks: {},

    navigationState: null,
    beforeNavigate: function (to, from) {
      if (navigatum.app) navigatum.app.error.msg = "";

      if (this.navigationState === "started") return; // Prevent duplicate calls
      this.navigationState = "started";

      document.getElementById("content").classList.remove("visible");
      document.getElementById("content").style.opacity = "0";
      document.getElementById("loading-page").classList.add("show");
      if (from.name !== null && window.history.saveCurrentViewState)
        window.history.saveCurrentViewState(); // Initial page load
    },
    afterNavigate: function (to, from) {
      if (this.navigationState === null) return;
      this.navigationState = null;

      navigatum.setUrl(); // sets only the og:url meta tag

      // This timeout is required because else the browser might skip to
      // transition if the change is too fast (if resources are in cache)
      window.setTimeout(function () {
        document.getElementById("content").classList.add("visible");
        document.getElementById("content").style.opacity = "";
        document.getElementById("loading-page").classList.remove("show");
      }, 5); // await at least one frame

      window.history.lastStateIndex = null; // Reset
    },

    // TODO: These are just helper functions and only cloneState is required
    // directly on pageload. Maybe we can move them somewhere else (but still in
    // the core code)
    cloneState: function (stateObj) {
      // cf. StackOverflow: https://stackoverflow.com/questions/728360/how-do-i-correctly-clone-a-javascript-object
      // State has to be serializable!
      if (stateObj == null || typeof stateObj !== "object") return stateObj;
      // Arrays are currently not cloned (TODO: is this required?)
      if (stateObj instanceof Array) {
        return stateObj;
      }
      if (stateObj instanceof Object) {
        const copy = {};

        stateObj.forEach((attr) => {
          if (
            attr !== "__ob__" && // stuff by vue, recursive!
            Object.prototype.hasOwnProperty.call(stateObj, attr) // see https://stackoverflow.com/q/39282873 why prototype
          )
            copy[attr] = this.cloneState(stateObj[attr]);
        });
        return copy;
      }
      console.error("failed to clone the state", stateObj);
      return {};
    },
    tryReuseViewState: function () {
      // Try to reuse the view state if there is one.
      if (
        window.history.states &&
        window.history.states[window.history.stateIndex][0].viewState
      ) {
        // We assume instance exists, because this should only be called
        // from a matched route
        const instance =
          navigatum.router.currentRoute.matched[0].instances.default;

        if (instance.state)
          this.applyState(
            window.history.states[window.history.stateIndex][0].viewState,
            instance.state
          );
        return true;
      }
      return false;
    },
    applyState: function (cacheStateObj, vueStateObj) {
      Object.keys(cacheStateObj).forEach((attr) => {
        if (cacheStateObj[attr] instanceof Object) {
          if (!(vueStateObj[attr] instanceof Object)) vueStateObj[attr] = {}; // value was null
          this.applyState(cacheStateObj[attr], vueStateObj[attr]);
        } else {
          vueStateObj[attr] = cacheStateObj[attr];
        }
      });
    },
    setTitle: function (name) {
      document.title = `${name} – NavigaTUM`;
      document
        .querySelector('meta[property="og:title"]')
        .setAttribute("content", name);
    },
    setDescription: function (description) {
      document
        .querySelector('meta[name="description"]')
        .setAttribute("content", description);
      document
        .querySelector('meta[property="og:description"]')
        .setAttribute("content", description);
    },
    setUrl: function () {
      document
        .querySelector('meta[property="og:url"]')
        .setAttribute("content", window.location.href);
    },
    // Settings are also stored in localStorage to detect when setting
    // a cookie did not work.
    setLang: function (lang) {
      localStorage.setItem("lang", lang);
      document.cookie = `lang=${lang};Max-Age=31536000;SameSite=Lax;Path=/* @echo app_prefix */`;
      window.location.reload(true);
    },
    setTheme: function (theme) {
      localStorage.setItem("theme", theme);
      document.cookie = `theme=${theme};Max-Age=31536000;SameSite=Lax;Path=/* @echo app_prefix */`;
      window.location.reload(true);
    },
  };
})();

navigatum.routes = [
  { path: "/", component: navigatum.getView("main") },
  {
    path: "/(view|campus|site|building|room)/:id",
    component: navigatum.getView("view"),
  },
  { path: "/search", component: navigatum.getView("search") },
  { path: "/about/:name", component: navigatum.getView("md") },
  { path: "/:catchAll(.*)", component: navigatum.getView("404") },
];
