import { createRouter, createWebHistory } from "vue-router";
import DashboardView from "@/views/DashboardView.vue";
import ProjectsView from "@/views/ProjectsView.vue";
import PhpView from "@/views/PhpView.vue";
import LogsView from "@/views/LogsView.vue";
import SettingsView from "@/views/SettingsView.vue";

export default createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/", name: "overview", component: DashboardView },
    { path: "/services", redirect: "/" },
    { path: "/projects", name: "projects", component: ProjectsView },
    { path: "/php", name: "php", component: PhpView },
    { path: "/logs", name: "logs", component: LogsView },
    { path: "/settings", name: "settings", component: SettingsView },
  ],
});
