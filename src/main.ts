import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import router from "./router";
import { bootTheme } from "@/lib/themes";
import { bootLocale } from "@/lib/i18n";
import "./style.css";

bootTheme();
bootLocale();

const app = createApp(App);
app.use(createPinia());
app.use(router);
app.mount("#app");
