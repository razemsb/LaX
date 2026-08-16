import { createRouter, createWebHistory } from "vue-router";
import DashboardView from "@/views/DashboardView.vue";
import ProjectsView from "@/views/ProjectsView.vue";
import PhpView from "@/views/PhpView.vue";
import LogsView from "@/views/LogsView.vue";
import SettingsView from "@/views/SettingsView.vue";

export default createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/", name: "Обзор", component: DashboardView },
    { path: "/services", redirect: "/" },
    { path: "/projects", name: "Проекты", component: ProjectsView },
    { path: "/php", name: "PHP", component: PhpView },
    { path: "/logs", name: "Логи", component: LogsView },
    { path: "/settings", name: "Настройки", component: SettingsView },
  ],
});
