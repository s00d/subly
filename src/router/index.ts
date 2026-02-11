import { createRouter, createWebHistory } from "vue-router";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      name: "dashboard",
      component: () => import("@/pages/DashboardPage.vue"),
    },
    {
      path: "/subscriptions",
      name: "subscriptions",
      component: () => import("@/pages/SubscriptionsPage.vue"),
    },
    {
      path: "/expenses",
      name: "expenses",
      component: () => import("@/pages/ExpensesPage.vue"),
    },
    {
      path: "/calendar",
      name: "calendar",
      component: () => import("@/pages/CalendarPage.vue"),
    },
    {
      path: "/settings",
      name: "settings",
      component: () => import("@/pages/SettingsPage.vue"),
    },
  ],
});

export default router;
