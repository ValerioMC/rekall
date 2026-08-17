import { createRouter, createWebHistory } from 'vue-router'

/**
 * The console (`/`) is still the one surface for doing work — this router exists for what sits
 * alongside it: browsing and shaping the catalog itself, which needs room the console's popover
 * never had. History mode, not hash, because `SinglePageApplicationRouting` on the server side
 * already forwards these exact paths to `index.html` and expects clean URLs.
 */
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
    }
  ]
})
