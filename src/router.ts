import { createRouter, createWebHashHistory } from "vue-router";
import LibraryView from "./views/LibraryView.vue";
import DetailView from "./views/DetailView.vue";
import ReaderView from "./views/ReaderView.vue";

export default createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", component: LibraryView },
    { path: "/detail/:id", component: DetailView, props: true },
    { path: "/reader/:id", component: ReaderView, props: true },
  ],
});