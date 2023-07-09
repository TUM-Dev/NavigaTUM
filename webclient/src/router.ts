import { createRouter, createWebHistory } from "vue-router";
import MainView from "@/pages/index.vue";
import NotFoundView from "@/pages/[...slug].vue";
import SearchView from "@/pages/search.vue";

const routes = [
  {
    component: MainView,
    name: "main",
    path: "/",
  },
  {
    component: () => import("@/pages/view/[id].vue"),
    name: "detail",
    path: "/:view(view|campus|site|building|room|poi)/:id",
  },
  {
    component: SearchView,
    name: "search",
    path: "/search",
  },
  {
    component: () => import("@/pages/api.vue"),
    name: "api",
    path: "/api",
  },
  {
    component: () => import("@/pages/about/[name].vue"),
    name: "about",
    path: "/about/:name",
  },
  {
    component: NotFoundView,
    name: "404",
    path: "/:catchAll(.*)*",
  },
];

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: routes,
  scrollBehavior(to, from, savedPosition) {
    // A resizeObserver is only necessary if the new position is > 0
    if (savedPosition && savedPosition.top > 0) {
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
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

router.afterEach(
  () => document.querySelector('meta[property="og:url"]')?.setAttribute("content", window.location.href),
);

export default router;
