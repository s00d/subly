import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import router from "./router";
import { setupI18n } from "./i18n";
import "./style.css";

async function bootstrap() {
  if (import.meta.env.DEV) {
    // Never let dev tooling block startup.
    try {
      const dev = await import("./dev/ipcFixtures");
      if ("__TAURI_INTERNALS__" in window) dev.installIpcRecorder();
      else await dev.installIpcMock();
    } catch (e) {
      console.warn("[ipc-fixtures] disabled:", e);
    }
  }
  const app = createApp(App);
  app.use(createPinia());

  const i18n = await setupI18n("en");
  app.use(i18n);

  app.use(router);
  app.mount("#app");
}

bootstrap();
