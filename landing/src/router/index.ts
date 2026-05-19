import { createRouter, createWebHistory } from "vue-router";
import { pageTitles } from "@/content/site.en";
import HomeView from "@/views/HomeView.vue";
import PrivacyView from "@/views/PrivacyView.vue";
import ChangelogView from "@/views/ChangelogView.vue";

export const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: "/", name: "home-en", component: HomeView, meta: { locale: "en", isHome: true } },
    { path: "/ru", name: "home-ru", component: HomeView, meta: { locale: "ru", isHome: true } },
    {
      path: "/privacy",
      name: "privacy",
      component: PrivacyView,
      meta: { title: pageTitles.privacy },
    },
    {
      path: "/changelog",
      name: "changelog",
      component: ChangelogView,
      meta: { title: pageTitles.changelog },
    },
  ],
  scrollBehavior(to) {
    if (to.hash) {
      return { el: to.hash, behavior: "smooth", top: 80 };
    }
    return { top: 0 };
  },
});
