/*
 * This is the first JS code executed for all views.
 */

// This is a wrapper around fetch that avoids duplicate requests if the
// same resource is requested another time before the first request has
// returned.
const cached_fetch = (function () {
    return {
        fetch: function(url, options) {
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
                                if (response.status == 404)
                                    throw new Error('${{_.core_js.error.404}}$');
                                else if (response.status == 500)
                                    throw new Error('${{_.core_js.error.500}}$');
                                else if (response.status == 503)
                                    throw new Error('${{_.core_js.error.503}}$');
                                else {
                                    const errorStatus = '${{_.core_js.error.status}}$';
                                    throw new Error(`${errorStatus}$${response.status}`);
                                }
                            }
                            navigatum.app.error.msg = null;
                            return options.as_text ? response.text() : response.json();
                        })
                        .catch((error) => {
                            let msg;
                            if (error instanceof TypeError) msg = '${{_.core_js.error.network}}$';
                            else msg = error.message;

                            if (!msg) msg = '${{_.core_js.error.unknown}}$';

                            console.warn('Error on fetch:');
                            console.log(error);

                            if (navigatum && navigatum.app) navigatum.app.error.msg = msg;

                            return null;
                        })
                        .then((data) => {
                            if (data !== null) this.cache[url] = data;
                            for (const i in this.promise_callbacks[url])
                                this.promise_callbacks[url][i](data);
                            delete this.promise_callbacks[url];
                        });
                }
            });
        },
        cache: {},
        promise_callbacks: {},
    };
})();

var navigatum = (function () {
    const api_base = '/* @echo api_prefix */';
    const cache = {};
    const get_data_resolve_callbacks = {};

    const views = {};
    const views_resolve_callbacks = {};
    let routes;

    const router = null;
    const app = null; // This is the Vue.js app

    /*
     * Most requests are just very simple GET requests.
     * This function is private and used by getData()
     */
    function GETRequest(url, onsuccess, onerror) {
        const req = new XMLHttpRequest();
        req.open('GET', this.api_base + url, true);
        req.onload = function () {
            onsuccess(this);
        };
        req.onerror = function () {
            onerror(this);
        };
        req.send();
    }

    function _modulePostInit(_this, name, c) {
        _this.modules.initialized[name] = c;
        if (name in _this.modules.loaded) delete _this.modules.loaded[name];

        for (const i in _this.module_promise_callbacks[name])
            _this.module_promise_callbacks[name][i](c);
        delete _this.module_promise_callbacks[name];
    }

    return {
        api_base: api_base,
        init: function() {
            // Init Vue.js
            this.router = new VueRouter({
                /* @if target="release" */
                mode: 'history',
                base: '/* @echo app_prefix */',
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
                    document.getElementById('content').scrollIntoView();

                    return { x: 0, y: 0, behavior: 'smooth' };
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
                el: '#app',
                data: {
                    search: {
                        focused: false,
                        keep_focus: false,
                        query: '',
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
                    searchfocus: function() {
                        this.search.focused = true;
                        this.search.autocomplete.highlighted = null;
                    },
                    searchblur: function() {
                        if (this.search.keep_focus) {
                            window.setTimeout(function () {
                                // This is relevant if the call is delayed and focused has
                                // already been disabled e.g. when clicking on an entry.
                                if (this.search.focused) document.getElementById('search').focus();
                            }, 0);
                            this.search.keep_focus = false;
                        } else {
                            this.search.focused = false;
                        }
                    },
                    searchinput: function(e) {
                        navigatum.getModule('autocomplete').then(function (c) {
                            c.oninput(e.srcElement.value);
                        });
                    },
                    searchkeydown: function(e) {
                        navigatum.getModule('autocomplete').then(function (c) {
                            c.onkeydown(e);
                        });
                    },
                    searchExpand: function(s) {
                        s.expanded = true;
                    },
                    searchGo: function(clean_query) {
                        if (this.search.query.length == 0) return;

                        navigatum.router.push(`/search?q=${this.search.query}`)
                                 .catch(() => { navigatum.afterNavigate() });
                        this.search.focused = false;
                        if (clean_query) {
                            this.search.query = '';
                            this.search.autocomplete.sections = [];
                        }
                        document.getElementById('search').blur();
                    },
                    searchGoTo: function(id, clean_query) {
                        // Catch is necessary because vue-router throws an error
                        // if navigation is aborted for some reason (e.g. the new
                        // url is the same or there is a loop in redirects)
                        navigatum.router.push(`/view/${id}`))
                                 .catch(() => { navigatum.afterNavigate() });
                        this.search.focused = false;
                        if (clean_query) {
                            this.search.query = '';
                            this.search.autocomplete.sections = [];
                        }
                        document.getElementById('search').blur();
                    },
                    setLang: navigatum.setLang,
                    setTheme: navigatum.setTheme,
                },
            });
        },
        cache: cache,
        /*
         * getData() either uses GETRequest() to retrieve the specified data
         * or loads it from its local cache. TODO: Update
         */
        getData: function(id, extended) {
            return new Promise((resolve) => {
                cached_fetch
                    .fetch(
                        `${this.api_base}get/${window.encodeURIComponent(id)}`,
                        { cache: 'force-cache' },
                    )
                    .then((data) => resolve(data));
            });
        },
        getBaseData: function(id) {
            return this.getData(id, false);
        },
        getExtendedData: function(id) {
            return this.getData(id, true);
        },
        setLocalStorageWithExpiry: function(key, value, ttl) {
            // ttl in hours
            const now = new Date();

            const item = {
                value: value,
                expiry: now.getTime() + ttl * 3.6e6,
            };
            localStorage.setItem(key, JSON.stringify(item));

            // "storage" usually fires only across tabs, this way we
            // force it to fire in this window as well
            const e = new Event('storage');
            window.dispatchEvent(e);
        },
        getLocalStorageWithExpiry: function(key, default_value=null) {
            const itemStr = localStorage.getItem(key);
            if (!itemStr) {
                return default_value;
            }
            const item = JSON.parse(itemStr);
            const now = new Date();
            if (now.getTime() > item.expiry) {
                localStorage.removeItem(key);
                return default_value;
            }
            return item.value;
        },
        removeLocalStorage: function(key) {
            localStorage.removeItem(key);
            const e = new Event('storage');
            window.dispatchEvent(e);
        },
        putData(id, data) {},
        /*
         * Views can be lazy loaded. Each view will call registerView() once it is
         * availabe. If the router requests a view with getView(), it can be directly
         * returned if it is already available. If not, it is retrieved and returned
         * as soon as it is availabe (views_resolve_callbacks stores the callbacks for
         * this).
         */
        // NOTE: This code doesn't use `this` because for some reason it doesn't work with IE
        registerView: function(name, component) {
            if (!(name in navigatum.views)) navigatum.views[name] = component;
            if (name in views_resolve_callbacks) {
                views_resolve_callbacks[name](component);
                delete views_resolve_callbacks[name];
            }
        },
        getView: function(name) {
            return function (resolve, reject) {
                if (name in navigatum.views) {
                    resolve(navigatum.views[name]);
                } else {
                    views_resolve_callbacks[name] = resolve;
                    window.setTimeout(function () {
                        if (name in views_resolve_callbacks) {
                            if (navigatum.app)
                                navigatum.app.error.msg = '${{_.core_js.error.view_load_timeout}}$';
                            reject('Load timed out');
                        }
                    }, 15000);
                }
            };
        },
        views: views,
        registerModule: function(name, c) {
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
        getModule: function(name, ...args) {
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
        beforeNavigate: function(to, from) {
            if (navigatum.app) navigatum.app.error.msg = '';

            if (this.navigationState === 'started') return; // Prevent duplicate calls
            this.navigationState = 'started';

            document.getElementById('content').classList.remove('visible');
            document.getElementById('content').style.opacity = '0';
            document.getElementById('loading-page').classList.add('show');
            if (from.name !== null && history.saveCurrentViewState) history.saveCurrentViewState(); // Initial page load
        },
        afterNavigate: function(to, from) {
            if (this.navigationState === null) return;
            this.navigationState = null;

            navigatum.setUrl(); // sets only the og:url meta tag

            // This timeout is required because else the browser might skip to
            // transition if the change is too fast (if resources are in cache)
            window.setTimeout(function () {
                document.getElementById('content').classList.add('visible');
                document.getElementById('content').style.opacity = '';
                document.getElementById('loading-page').classList.remove('show');
            }, 5); // await at least one frame

            history.lastStateIndex = null; // Reset
        },

        // TODO: These are just helper functions and only cloneState is required
        // directly on pageload. Maybe we can move them somewhere else (but still in
        // the core code)
        cloneState: function(state_obj) {
            // cf. StackOverflow: https://stackoverflow.com/questions/728360/how-do-i-correctly-clone-a-javascript-object
            // State has to be serializable!
            if (state_obj == null || typeof state_obj !== 'object') return state_obj;
            // Arrays are currently not cloned (TODO: is this required?)
            if (state_obj instanceof Array) {
                return state_obj;
            }
            if (state_obj instanceof Object) {
                const copy = {};
                for (const attr in state_obj) {
                    if (
                        attr != '__ob__' && // stuff by vue, recursive!
                        state_obj.hasOwnProperty(attr)
                    )
                        copy[attr] = this.cloneState(state_obj[attr]);
                }
                return copy;
            }
        },
        tryReuseViewState: function() {
            // Try to reuse the view state if there is one.
            if (history.states && history.states[history.stateIndex][0].viewState) {
                // We assume instance exists, because this should only be called
                // from a matched route
                const instance = navigatum.router.currentRoute.matched[0].instances.default;

                if (instance.state)
                    this.applyState(
                        history.states[history.stateIndex][0].viewState,
                        instance.state,
                    );
                return true;
            }
            return false;
        },
        applyState: function(cache_state_obj, vue_state_obj) {
            for (const attr in cache_state_obj) {
                if (cache_state_obj[attr] instanceof Object) {
                    if (!(vue_state_obj[attr] instanceof Object)) vue_state_obj[attr] = {}; // value was null
                    this.applyState(cache_state_obj[attr], vue_state_obj[attr]);
                } else {
                    vue_state_obj[attr] = cache_state_obj[attr];
                }
            }
        },
        setTitle: function(name) {
            document.title = `${name} – NavigaTUM`;
            document.querySelector('meta[property="og:title"]').setAttribute('content', name);
        },
        setDescription: function(description) {
            document.querySelector('meta[name="description"]').setAttribute('content', description);
            document
                .querySelector('meta[property="og:description"]')
                .setAttribute('content', description);
        },
        setUrl: function() {
            document
                .querySelector('meta[property="og:url"]')
                .setAttribute('content', window.location.href);
        },
        // Settings are also stored in localStorage to detect when setting
        // a cookie did not work.
        setLang: function(lang) {
            localStorage.setItem('lang', lang);
            document.cookie = `lang=${lang};Max-Age=31536000;SameSite=Lax;Path=/* @echo app_prefix */`;
            window.location.reload(true);
        },
        setTheme: function(theme) {
            localStorage.setItem('theme', theme);
            document.cookie = `theme=${theme};Max-Age=31536000;SameSite=Lax;Path=/* @echo app_prefix */`;
            window.location.reload(true);
        },
    };
})();

navigatum.routes = [
    { path: '/', component: navigatum.getView('main') },
    { path: '/(view|campus|site|building|room)/:id', component: navigatum.getView('view') },
    { path: '/search', component: navigatum.getView('search') },
    { path: '/about/:name', component: navigatum.getView('md') },
    { path: '/:catchAll(.*)', component: navigatum.getView('404') },
];
