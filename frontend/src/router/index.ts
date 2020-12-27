import Vue from "vue";
import VueRouter, { RouteConfig } from "vue-router";

Vue.use(VueRouter);

const routes: Array<RouteConfig> = [
  {
    path: "/",
    name: "GP",
    component: () =>
      import(/* webpackChunkName: "gp" */ "../views/GaussianProcess.vue"),
  },
  {
    path: "/graphics",
    name: "graphics",
    component: () =>
      import(/*webpackChunkName: "graphics"*/ "../views/Graphics.vue"),
  },
];

const router = new VueRouter({
  mode: "history",
  base: process.env.NODE_ENV === "development" ? "/" : "/playground/",
  routes,
});

export default router;
