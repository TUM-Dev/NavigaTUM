import { createRouter, createWebHistory } from "vue-router";
import MainView from "./views/MainView.vue";
import NotFoundView from "./views/NotFoundView.vue";
import SearchView from "./views/SearchView.vue";

const routes = [
  { path: "/", name: "main", component: MainView },
  {
    path: "/:view(view|campus|site|building|room|poi)/:id",
    name: "detail",
    component: () => import("./views/DetailsView.vue"),
  },
  { path: "/search", name: "search", component: SearchView },
  {
    path: "/api",
    name: "api",
    component: () => import("./views/APIView.vue"),
  },
  {
    path: "/about/:name",
    name: "about",
    component: () => import("./views/AboutView.vue"),
  },
  {
    path: "/:catchAll(.*)",
    name: "404",
    component: NotFoundView,
  },
];

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: routes,
  scrollBehavior(to, from, savedPosition) {
    // A resizeObserver is only necessary if the new position is > 0
    if (savedPosition && savedPosition.top > 0) {
      return new Promise((resolve, reject) => {
        // ResizeObserver similar to https://stackoverflow.com/a/72944150
        // The body getBoundingClientRect().y is necessary becase the body is shifted
        // by the top margin of the first content element (#content-header) and
        // document.body.clientTop does always give zero.
        const requiredHeight =
          savedPosition.top + document.documentElement.clientHeight - document.body.getBoundingClientRect().y;
        const resizeObserver = new ResizeObserver((entries) => {
          if (entries[0].target.clientHeight >= requiredHeight) {
            resizeObserver.disconnect();
            resolve(savedPosition);
          }
        });
        resizeObserver.observe(document.body);
      });
    }

    // Just returning (0, 0) does not work when the new page is
    // the same component, and it got so small, that the current
    // position is now the bottom of the new page.
    // For this reason this extra call.
    document.getElementById("content")?.scrollIntoView();

    return { top: 0, left: 0, behavior: "smooth" };
  },
});

export default router;
