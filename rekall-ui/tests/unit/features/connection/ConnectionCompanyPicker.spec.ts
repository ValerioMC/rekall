import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import { createMemoryHistory, createRouter } from 'vue-router'
import ConnectionCompanyPicker from '@/features/connection/components/ConnectionCompanyPicker.vue'
import type { CompanyId } from '@/model/branded'
import { ACME, GLOBEX, aBinding } from '../../support/connectionFixture'

const COMPANIES = [
  { id: ACME, name: 'Acme' },
  { id: GLOBEX, name: 'Globex' },
  { id: 'company-initech' as CompanyId, name: 'Initech' }
]

function mountPicker(props: Record<string, unknown>) {
  const router = createRouter({ history: createMemoryHistory(), routes: [{ path: '/:p(.*)*', component: { template: '<div />' } }] })
  return mount(ConnectionCompanyPicker, { props: { companies: COMPANIES, modelValue: [], ...props }, global: { plugins: [router] } })
}

describe('ConnectionCompanyPicker', () => {
  it('checks the companies the connection is shared with and emits the new set when one is toggled', async () => {
    const wrapper = mountPicker({ modelValue: [ACME] })
    const boxes = wrapper.findAll('input[type="checkbox"]')

    expect(boxes.map((box) => (box.element as HTMLInputElement).checked)).toEqual([true, false, false])
    await boxes[1]?.setValue(true)
    expect(wrapper.emitted('update:modelValue')).toEqual([[[ACME, GLOBEX]]])
    await boxes[0]?.setValue(false)
    expect(wrapper.emitted('update:modelValue')?.[1]).toEqual([[]])
  })

  it('narrows the list with the search', async () => {
    const wrapper = mountPicker({})

    await wrapper.get('[data-testid="company-search"]').setValue('glo')

    expect(wrapper.findAll('[data-testid="company-option"]').map((option) => option.text())).toEqual(['Globex'])
  })

  it('locks a company whose projects still bind the connection and names them', () => {
    const blockedBy = new Map([[ACME, [aBinding('billing', ACME, { title: 'Billing' }), aBinding('portal', ACME, { title: 'Portal' })]]])
    const wrapper = mountPicker({ modelValue: [ACME], blockedBy })
    const [acme, globex] = wrapper.findAll('input[type="checkbox"]')

    expect((acme?.element as HTMLInputElement).disabled).toBe(true)
    expect((globex?.element as HTMLInputElement).disabled).toBe(false)
    const reason = wrapper.get('[data-testid="company-locked"]')
    expect(reason.text()).toContain('2 projects of Acme bind this connection')
    expect(reason.findAll('a').map((link) => link.text())).toEqual(['Billing', 'Portal'])
  })
})
