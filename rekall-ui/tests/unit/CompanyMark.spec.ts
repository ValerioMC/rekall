import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import CompanyMark from '@/components/ui/CompanyMark.vue'
import { companyInitial } from '@/common/company/logo-image-url'

const COMPANY = '9f1c2d3e-4b5a-4c6d-8e7f-001122334455'

describe('CompanyMark', () => {
  it("wears the company's initial on its identity hue when there is no logo", () => {
    const wrapper = mount(CompanyMark, { props: { companyId: COMPANY, name: 'vforge', size: 24 } })

    const mark = wrapper.get('[data-testid="company-mark-initial"]')
    expect(mark.text()).toBe('V')
    expect(mark.attributes('style')).toContain('var(--color-identity-')
    expect(wrapper.find('img').exists()).toBe(false)
  })

  it('shows the logo at its own size with one candidate per density in device pixels', () => {
    const wrapper = mount(CompanyMark, {
      props: { companyId: COMPANY, name: 'vforge', imageVersion: '2026-10-09T10:00:00Z', size: 24 }
    })

    const image = wrapper.get('img')
    expect(image.attributes('src')).toContain(`/api/companies/${COMPANY}/logo-image?size=24`)
    expect(image.attributes('srcset')).toMatch(/size=24.* 1x, .*size=48.* 2x, .*size=72.* 3x/)
    expect(image.attributes('alt')).toBe('')
  })

  it('shows a file just picked in place of the stored logo', () => {
    const wrapper = mount(CompanyMark, {
      props: { companyId: COMPANY, name: 'vforge', imageVersion: 'v1', previewSrc: 'blob:preview', size: 48 }
    })

    expect(wrapper.get('img').attributes('src')).toBe('blob:preview')
    expect(wrapper.get('img').attributes('srcset')).toBeUndefined()
  })
})

describe('companyInitial', () => {
  it('takes the first whole character, upper-cased, past leading spaces', () => {
    expect(companyInitial('  ärzte')).toBe('Ä')
    expect(companyInitial('😀 smiles')).toBe('😀')
  })
})
