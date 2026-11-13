import { createRouter, createWebHistory } from 'vue-router'

export const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', name: 'work', component: () => import('@/App.vue') },
    { path: '/work', redirect: '/' },
    { path: '/work/board', name: 'work-board', component: () => import('@/App.vue') },
    { path: '/task/:id/:tab?', name: 'task', component: () => import('@/App.vue') },
    { path: '/console', name: 'console', component: () => import('@/App.vue') },
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
      path: '/companies/:id/connections',
      redirect: (to) => ({ name: 'connections', query: { company: String(to.params.id) } })
    },
    {
      path: '/connections/:id?',
      name: 'connections',
      component: () => import('@/features/connection/components/ConnectionsPage.vue')
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
      path: '/security',
      name: 'security',
      component: () => import('@/features/vulnerability/components/SecurityPage.vue')
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
