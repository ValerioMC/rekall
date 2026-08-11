import { createRouter, createWebHistory } from 'vue-router'

/**
 * Routes are lazily imported so a screen's code is fetched when it is first opened, and
 * `props: true` hands the params to the component as typed props rather than leaving it to
 * read them off the route.
 */
const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', redirect: '/schema' },
    {
      path: '/schema',
      name: 'schema-list',
      component: () => import('@/views/schema/SchemaListView.vue')
    },
    {
      path: '/schema/:id',
      name: 'schema-detail',
      component: () => import('@/views/schema/SchemaDetailView.vue'),
      props: true
    },
    { path: '/plan', name: 'plan', component: () => import('@/views/plan/PlanView.vue') },
    {
      path: '/data',
      name: 'data-root',
      component: () => import('@/views/data/DataListView.vue')
    },
    {
      path: '/data/:entity',
      name: 'data-list',
      component: () => import('@/views/data/DataListView.vue'),
      props: true
    },
    {
      path: '/data/:entity/:id',
      name: 'data-detail',
      component: () => import('@/views/data/DataDetailView.vue'),
      props: true
    },
    { path: '/search', name: 'search', component: () => import('@/views/search/SearchView.vue') }
  ]
})

export default router
