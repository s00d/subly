import { createRouter, createWebHistory, type RouteRecordRaw } from "vue-router";
import { docPageTitles } from "@/content/docMeta";
import { homePathFor, SUPPORT_LOCALES } from "@/content/locales";
import HomeView from "@/views/HomeView.vue";
import PrivacyView from "@/views/PrivacyView.vue";
import ChangelogView from "@/views/ChangelogView.vue";

const homeRoutes: RouteRecordRaw[] = SUPPORT_LOCALES.map((locale) => ({
  path: homePathFor(locale),
  name: locale === "en" ? "home-en" : `home-${locale}`,
  component: HomeView,
  meta: { locale, isHome: true },
}));

export const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    ...homeRoutes,
    {
      path: "/privacy",
      name: "privacy",
      component: PrivacyView,
      meta: { title: docPageTitles.privacy },
    },
    {
      path: "/changelog",
      name: "changelog",
      component: ChangelogView,
      meta: { title: docPageTitles.changelog },
    },
  ],
  scrollBehavior(to) {
    if (to.hash) {
      return { el: to.hash, behavior: "smooth", top: 80 };
    }
    return { top: 0 };
  },
});
