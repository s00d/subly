import { createRouter, createWebHistory } from "vue-router";
import HomeView from "@/views/HomeView.vue";
import PrivacyView from "@/views/PrivacyView.vue";
import ChangelogView from "@/views/ChangelogView.vue";

export const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: "/", name: "home", component: HomeView },
    {
      path: "/privacy",
      name: "privacy",
      component: PrivacyView,
      meta: { title: "Privacy Policy — Subly" },
    },
    {
      path: "/changelog",
      name: "changelog",
      component: ChangelogView,
      meta: { title: "Changelog — Subly" },
    },
  ],
  scrollBehavior(to) {
    if (to.hash) {
      return { el: to.hash, behavior: "smooth", top: 80 };
    }
    return { top: 0 };
  },
});
