import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import ConnectionDeleteDialog from '@/features/connection/components/ConnectionDeleteDialog.vue'
import { ACME, GLOBEX, aBinding, aConnection, aDetail } from '../../support/connectionFixture'

function mountDialog(detail: ReturnType<typeof aDetail> | null) {
  const connection = aConnection('platform', { companyIds: [ACME, GLOBEX] })
  return mount(ConnectionDeleteDialog, { props: { connection, detail }, attachTo: document.body })
}

describe('ConnectionDeleteDialog', () => {
  it('states the blast radius first and repeats it on the destructive button', () => {
    const connection = aConnection('platform', { companyIds: [ACME, GLOBEX] })
    const wrapper = mountDialog(
      aDetail(connection, [aBinding('billing', ACME), aBinding('portal', ACME, { bindingLabel: 'portal-2' }), aBinding('billing', ACME, { bindingLabel: 'billing-2' })])
    )
    const text = document.body.textContent ?? ''

    expect(text).toContain('Delete The platform?')
    expect(text).toContain('Shared with 2 companies.')
    expect(text).toContain('2 projects lose it, with 3 bindings and their targets.')
    expect(text).toContain('Its stored credential is removed from this machine.')
    expect(text).toContain('Delete and unbind 2 projects')
    wrapper.unmount()
  })

  it('is a plain confirm when no project binds the connection', () => {
    const connection = aConnection('platform')
    const wrapper = mountDialog(aDetail(connection))

    expect(document.body.textContent).toContain('No project binds it.')
    expect(document.body.textContent).toContain('Delete connection')
    expect(document.body.textContent).not.toContain('unbind')
    wrapper.unmount()
  })

  it('emits confirm and cancel', async () => {
    const wrapper = mountDialog(null)
    const buttons = Array.from(document.body.querySelectorAll('button'))

    buttons.find((button) => button.textContent?.includes('Delete connection'))?.click()
    buttons.find((button) => button.textContent?.includes('Keep it'))?.click()

    expect(wrapper.emitted('confirm')).toHaveLength(1)
    expect(wrapper.emitted('cancel')).toHaveLength(1)
    wrapper.unmount()
  })
})
