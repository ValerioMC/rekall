import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { defineComponent, h } from 'vue'
import { createPinia, setActivePinia, type Pinia } from 'pinia'
import { createRouter, createWebHistory, RouterView, type Router } from 'vue-router'
import { useConsoleLocation } from '@/composables/useConsoleLocation'
import { useConsoleStore } from '@/stores/console.store'
import type { TaskId } from '@/model/branded'
import { installFakeBrowserHistory } from './support/fakeBrowserHistory'
import { taskFixture } from './support/taskFixture'

/**
 * The console's moves become history entries, so going back walks tasks and panes. Runs on a
 * web history over a fake browser stack, because jsdom never fires `popstate` on its own.
 */
const report = 't1' as TaskId
const retry = 't2' as TaskId

const ConsoleHost = defineComponent({
  setup() {
    useConsoleLocation()
    return () => h('div')
  }
})

let router: Router
let pinia: Pinia
let store: ReturnType<typeof useConsoleStore>
let wrapper: VueWrapper | null = null
let restoreHistory: () => void = () => undefined

async function openConsoleAt(url: string): Promise<void> {
  await router.push(url)
  await router.isReady()
  wrapper = mount(RouterView, { global: { plugins: [router, pinia] } })
  await flushPromises()
}

async function settle(): Promise<void> {
  await flushPromises()
  await flushPromises()
}

beforeEach(() => {
  restoreHistory = installFakeBrowserHistory()
  pinia = createPinia()
  setActivePinia(pinia)
  store = useConsoleStore()
  store.tasks = [taskFixture(report), taskFixture(retry)]
  store.isLoading = false
  router = createRouter({
    history: createWebHistory(),
    routes: [
      { path: '/', name: 'console', component: ConsoleHost },
      { path: '/projects', name: 'projects', component: { render: () => h('div') } }
    ]
  })
})

afterEach(() => {
  wrapper?.unmount()
  wrapper = null
  restoreHistory()
})

describe('the console location in the history', () => {
  it('writes what a bare link opens on into the entry, without adding one', async () => {
    store.selectTask(report)
    await openConsoleAt('/')

    expect(router.currentRoute.value.query).toEqual({ mode: 'tasks', task: report })
    expect(window.history.length).toBe(1)
  })

  it('goes back through tasks and panes, and forward again', async () => {
    await openConsoleAt('/')
    store.selectTask(report)
    await settle()
    store.openSteps()
    await settle()
    store.selectTask(retry)
    await settle()

    router.back()
    await settle()
    expect([store.selectedTaskId, store.paneFocus]).toEqual([report, 'steps'])

    router.back()
    await settle()
    expect([store.selectedTaskId, store.paneFocus]).toEqual([report, 'note'])

    router.back()
    await settle()
    expect(store.selectedTaskId).toBeNull()

    router.forward()
    await settle()
    expect(store.selectedTaskId).toBe(report)
    expect(window.history.length).toBe(4)
  })

  it('rewrites an entry whose task was deleted since, instead of adding one', async () => {
    await openConsoleAt('/')
    store.selectTask(report)
    await settle()
    store.selectTask(retry)
    await settle()
    store.tasks = [taskFixture(retry)]

    router.back()
    await settle()

    expect(store.selectedTaskId).toBeNull()
    expect(router.currentRoute.value.query).toEqual({ mode: 'tasks' })
    expect(window.history.length).toBe(3)
  })

  it('reopens the location of a reloaded entry once the catalog has arrived', async () => {
    store.isLoading = true
    await openConsoleAt(`/?mode=tasks&task=${retry}&pane=wrapup`)
    expect(store.selectedTaskId).toBeNull()

    store.isLoading = false
    await settle()

    expect([store.selectedTaskId, store.paneFocus]).toEqual([retry, 'wrapup'])
  })

  it('records nothing while the reader is on another screen', async () => {
    await openConsoleAt('/')
    await router.push('/projects')
    await settle()

    store.selectTask(report)
    await settle()

    expect(router.currentRoute.value.name).toBe('projects')
    expect(window.history.length).toBe(2)
  })
})
