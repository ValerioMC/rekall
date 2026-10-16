import { createRouter, createWebHistory } from 'vue-router'

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', name: 'console', component: () => import('@/App.vue') },
    {
      path: '/projects',
      name: 'projects',
      component: () => import('@/features/catalog/components/ProjectListPage.vue')
    },
    {
      path: '/projects/:id',
      name: 'project-detail',
      component: () => import('@/features/catalog/components/ProjectDetailPage.vue'),
      props: true
    },
    {
      path: '/companies',
      name: 'companies',
      component: () => import('@/features/catalog/components/CompanyListPage.vue')
    },
    {
      path: '/report',
      name: 'report',
      component: () => import('@/features/time/components/ReportPage.vue')
    },
    {
      path: '/diagrams',
      name: 'diagrams',
      component: () => import('@/features/diagram/components/DiagramsPage.vue')
    },
    {
      path: '/claude',
      name: 'claude',
      component: () => import('@/features/claude-resources/components/ClaudeResourcesPage.vue')
    },
    {
      path: '/calendar',
      name: 'calendar',
      component: () => import('@/features/time/components/CalendarPage.vue')
    }
  ]
})
