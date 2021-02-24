import Vue from "vue";
import VueRouter, { RouteConfig } from "vue-router";

Vue.use(VueRouter);

const routes: Array<RouteConfig> = [
  {
    path: "/",
    name: "GP",
    // route level code-splitting
    // this generates a separate chunk (about.[hash].js) for this route
    // which is lazy-loaded when the route is visited.
    component: () =>
      import(/* webpackChunkName: "about" */ "../views/GaussianProcess.vue")
  },
  {
    path: "/anyonethere",
    name: "ANYONETHERE",
    component: () => import("../views/AnyoneThere.vue"),
  }
];

const router = new VueRouter({
  mode: "history",
  base: process.env.NODE_ENV === "development" ? "/" : "/playground/",
  routes
});

export default router;
