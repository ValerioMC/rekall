import { createRouter, createWebHistory } from 'vue-router'

/**
 * Routes are lazily imported so a screen's code is fetched when it is first opened, and
 * `props: true` hands the params to the component as typed props rather than leaving it to
 * read them off the route.
 */
const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', redirect: '/projects' },
    { path: '/projects', name: 'projects', component: () => import('@/views/ProjectsView.vue') },
    {
      path: '/projects/:id',
      name: 'project-detail',
      component: () => import('@/views/ProjectDetailView.vue'),
      props: true
    },
    {
      path: '/tasks/:id',
      name: 'task-detail',
      component: () => import('@/views/TaskDetailView.vue'),
      props: true
    },
    {
      path: '/environments',
      name: 'environments',
      component: () => import('@/views/EnvironmentsView.vue')
    },
    { path: '/search', name: 'search', component: () => import('@/views/SearchView.vue') }
  ]
})

export default router
