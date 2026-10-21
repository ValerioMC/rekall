import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import * as api from '@/features/connection/api/connections.api'
import KubeconfigImport from '@/features/connection/components/KubeconfigImport.vue'
import type { CredentialDraft, KubeContextOption } from '@/features/connection/model/connection'

vi.mock('@/features/connection/api/connections.api')

type KubeDraft = Extract<CredentialDraft, { kind: 'KUBERNETES' }>

function option(name: string, over: Partial<KubeContextOption> = {}): KubeContextOption {
  return {
    name,
    server: 'https://k8s.example.com',
    namespace: null,
    userKind: 'token',
    supported: true,
    refusal: null,
    ...over
  }
}

function mountImport(draft: KubeDraft) {
  return mount(KubeconfigImport, {
    props: {
      modelValue: draft,
      'onUpdate:modelValue': async (next: KubeDraft) => {
        await wrapper.setProps({ modelValue: next })
      }
    }
  })
}

let wrapper: ReturnType<typeof mountImport>

describe('KubeconfigImport', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetAllMocks()
  })

  it('lists the contexts, keeps a refused one out of reach and lets the person choose another', async () => {
    vi.mocked(api.previewKubeconfig).mockResolvedValue([
      option('prod'),
      option('staging'),
      option('cloud', {
        userKind: 'exec',
        supported: false,
        refusal: "Context 'cloud' cannot be imported: its user runs a command."
      })
    ])
    wrapper = mountImport({ kind: 'KUBERNETES', kubeconfig: 'apiVersion: v1', context: null })

    await wrapper.get('[data-testid="read-contexts"]').trigger('click')
    await flushPromises()

    const cloud = wrapper.get('[data-testid="context-cloud"]')
    expect(cloud.attributes('disabled')).toBeDefined()
    expect(cloud.text()).toContain('runs a command')
    expect(wrapper.props('modelValue').context).toBeNull()

    await wrapper.get('[data-testid="context-staging"]').trigger('click')

    expect(wrapper.props('modelValue').context).toBe('staging')
    expect(wrapper.get('[data-testid="context-staging"]').attributes('aria-checked')).toBe('true')
  })

  it('chooses the only importable context by itself', async () => {
    vi.mocked(api.previewKubeconfig).mockResolvedValue([
      option('prod'),
      option('cloud', { supported: false, refusal: 'no' })
    ])
    wrapper = mountImport({ kind: 'KUBERNETES', kubeconfig: 'apiVersion: v1', context: null })

    await wrapper.get('[data-testid="read-contexts"]').trigger('click')
    await flushPromises()

    expect(wrapper.props('modelValue').context).toBe('prod')
  })

  it('says why a kubeconfig cannot be read and offers nothing to choose', async () => {
    vi.mocked(api.previewKubeconfig).mockRejectedValue(
      new Error('The kubeconfig is not a valid kubeconfig at line 3, column 1.')
    )
    wrapper = mountImport({ kind: 'KUBERNETES', kubeconfig: 'broken: [', context: null })

    await wrapper.get('[data-testid="read-contexts"]').trigger('click')
    await flushPromises()

    expect(wrapper.get('[data-testid="kubeconfig-failure"]').text()).toContain('line 3')
    expect(wrapper.find('[role="radiogroup"]').exists()).toBe(false)
  })

  it('a new paste drops the context chosen for the old one', async () => {
    wrapper = mountImport({ kind: 'KUBERNETES', kubeconfig: 'old', context: 'prod' })

    await wrapper.get('[data-testid="kubeconfig-text"]').setValue('new')

    expect(wrapper.props('modelValue')).toEqual({ kind: 'KUBERNETES', kubeconfig: 'new', context: null })
  })
})
