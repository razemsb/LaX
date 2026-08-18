import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import router from "./router";
import { bootTheme } from "@/lib/themes";
import "./style.css";

bootTheme();

const app = createApp(App);
app.use(createPinia());
app.use(router);
app.mount("#app");
