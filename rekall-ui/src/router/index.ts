import { createRouter, createWebHistory } from 'vue-router'

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', name: 'console', component: () => import('@/App.vue') },
    {
      path: '/projects',
      name: 'projects',
      component: () => import('@/components/catalog/ProjectListPage.vue')
    },
    {
      path: '/projects/:id',
      name: 'project-detail',
      component: () => import('@/components/catalog/ProjectDetailPage.vue'),
      props: true
    },
    {
      path: '/companies',
      name: 'companies',
      component: () => import('@/components/catalog/CompanyListPage.vue')
    },
    {
      path: '/calendar',
      name: 'calendar',
      component: () => import('@/components/calendar/CalendarPage.vue')
    }
  ]
})
