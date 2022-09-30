import { createRouter, createWebHistory } from "vue-router";
import MainView from "./views/MainView.vue";
import NotFoundView from "./views/NotFoundView.vue";
import SearchView from "./views/SearchView.vue";

const routes = [
  { path: "/", name: "main", component: MainView },
  { path: "/view/:id",
    alias: ["/campus/:id", "/site/:id", "/building/:id", "/room/:id"],
    name: "detail",
    component: () => import("./views/DetailsView.vue") },
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
    if (savedPosition) {
      return savedPosition;
    }
    // Just returning (0, 0) does not work when the new page is
    // the same component, and it got so small, that the current
    // position is now the bottom of the new page.
    // For this reason this extra call.
    document.getElementById("content")?.scrollIntoView();

    return { x: 0, y: 0, behavior: "smooth" };
  },
});

export default router;
