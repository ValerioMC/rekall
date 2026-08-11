import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import { h } from 'vue'
import AppField from '@/components/ui/AppField.vue'
import AppInput from '@/components/ui/AppInput.vue'

describe('AppField', () => {
  it('associates its label with the control in its slot', () => {
    const wrapper = mount(AppField, {
      props: { label: 'Table name' },
      slots: {
        default: (slotProps: { fieldId: string }) =>
          h(AppInput, { id: slotProps.fieldId, modelValue: '' })
      }
    })

    const forAttribute = wrapper.find('label').attributes('for')
    const inputId = wrapper.find('input').attributes('id')

    expect(forAttribute).toBeTruthy()
    expect(inputId).toBe(forAttribute)
  })

  it('points the control at its hint through aria-describedby', () => {
    const wrapper = mount(AppField, {
      props: { label: 'Table name', hint: 'Cannot be changed later' },
      slots: {
        default: (slotProps: { fieldId: string; describedBy?: string }) =>
          h(AppInput, { id: slotProps.fieldId, describedBy: slotProps.describedBy, modelValue: '' })
      }
    })

    const describedBy = wrapper.find('input').attributes('aria-describedby')

    expect(describedBy).toBeTruthy()
    expect(wrapper.find(`#${describedBy}`).text()).toBe('Cannot be changed later')
  })

  it('announces a required field to a screen reader, not only with an asterisk', () => {
    const wrapper = mount(AppField, { props: { label: 'Description', required: true } })

    expect(wrapper.find('.sr-only').text()).toBe('required')
    expect(wrapper.find('[aria-hidden=true]').text()).toBe('*')
  })

  it('renders an error in place of the hint and marks it as an alert', () => {
    const wrapper = mount(AppField, {
      props: { label: 'Name', hint: 'A hint', error: 'Already taken' }
    })

    expect(wrapper.find('[role=alert]').text()).toBe('Already taken')
    expect(wrapper.text()).not.toContain('A hint')
  })
})
