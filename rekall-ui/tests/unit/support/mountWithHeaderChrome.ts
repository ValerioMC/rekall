import { DOMWrapper, mount, type MountingOptions } from '@vue/test-utils'
import type { Component } from 'vue'

/**
 * The real app renders `#app-header-extra` once, in `AppHeaderChrome`, and every screen
 * teleports its header content into it. Mounted alone in a test, a screen has nowhere to
 * teleport to, so this plants that target in the document before mounting, then routes the
 * usual query methods through `document.body` — the one ancestor the mounted component and
 * its teleported content share — so `find`/`findAll`/`get`/`text`/`html` keep seeing it.
 */
export function mountWithHeaderChrome(component: Component, options: MountingOptions<never> = {}) {
  document.body.innerHTML = '<div id="app-header-extra"></div>'
  const wrapper = mount(component, { ...options, attachTo: document.body })
  const body = new DOMWrapper(document.body)
  return Object.assign(wrapper, {
    find: body.find.bind(body),
    findAll: body.findAll.bind(body),
    get: body.get.bind(body),
    text: body.text.bind(body),
    html: body.html.bind(body)
  })
}
