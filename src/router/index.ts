import { createRouter, createWebHashHistory, RouteRecordRaw } from 'vue-router';
import DashboardView from '../views/DashboardView.vue';
import SettingsView from '../views/SettingsView.vue';
import ProjectsView from '../views/ProjectsView.vue';

const routes: Array<RouteRecordRaw> = [
  {
    path: '/',
    redirect: '/dashboard',
  },
  {
    path: '/dashboard',
    name: 'dashboard',
    component: DashboardView,
  },
  {
    path: '/settings',
    name: 'settings',
    component: SettingsView,
  },
  {
    path: '/projects',
    name: 'projects',
    component: ProjectsView,
  },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
