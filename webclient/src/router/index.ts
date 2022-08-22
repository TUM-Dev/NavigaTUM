import { createRouter, createWebHistory } from "vue-router";
import MainView from "../views/MainView.vue";
import NotFoundView from "../views/NotFoundView.vue";
import SearchView from "../views/SearchView.vue";

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {path: "/", name: "main", component: MainView,},
  {
    path: "/(view|campus|site|building|room)/:id",
    name: "detail",
    component: ()=> import("../views/DetailsView.vue"),
  },
  { path: "/search",
    name: "search",
    component: SearchView },
  { path: "/api",
    name: "api",
    component: ()=> import("../views/APIView.vue"), },
  { path: "/about/:name",
    name: "about",
    component: ()=> import("../views/AboutView.vue"), },
  { path: "/:catchAll(.*)", component: NotFoundView},
  ],
});

export default router;
