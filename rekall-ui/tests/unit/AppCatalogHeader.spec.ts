import { afterEach, describe, expect, it } from 'vitest'
import { defineComponent, h } from 'vue'
import AppCatalogHeader from '@/features/catalog/components/AppCatalogHeader.vue'
import { mountWithHeaderChrome } from './support/mountWithHeaderChrome'

const Host = defineComponent({ render: () => h(AppCatalogHeader, { title: 'Diagrams' }) })

describe('AppCatalogHeader', () => {
  afterEach(() => {
    document.body.innerHTML = ''
  })

  it('teleports exactly two cells so the tabs keep the same place as on every screen', () => {
    mountWithHeaderChrome(Host)

    const slot = document.getElementById('app-header-extra')

    expect(slot?.children).toHaveLength(2)
    expect(slot?.children[0]?.textContent).toContain('Diagrams')
  })
})
