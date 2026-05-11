import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import router from "./router";
import { setupI18n } from "./i18n";
import "./style.css";

async function bootstrap() {
  const app = createApp(App);
  app.use(createPinia());

  const i18n = await setupI18n("en");
  app.use(i18n);

  app.use(router);
  app.mount("#app");
}

bootstrap();
