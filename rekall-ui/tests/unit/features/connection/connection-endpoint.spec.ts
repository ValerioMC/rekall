import { describe, expect, it } from 'vitest'
import { endpointOf, hostOf } from '@/features/connection/lib/connection-endpoint'

describe('hostOf', () => {
  it('keeps the host and port of a URL', () => {
    expect(hostOf('https://gitlab.example.com:8443/api/v4')).toBe('gitlab.example.com:8443')
  })

  it('returns text that is not a URL unchanged', () => {
    expect(hostOf('gitlab.local')).toBe('gitlab.local')
  })
})

describe('endpointOf', () => {
  it('names a GitLab instance by its host', () => {
    expect(endpointOf({ kind: 'GITLAB', baseUrl: 'https://gitlab.example.com', kubeContext: null })).toBe(
      'gitlab.example.com'
    )
  })

  it('names a cluster by its context and server', () => {
    expect(
      endpointOf({ kind: 'KUBERNETES', baseUrl: 'https://10.0.0.14:6443', kubeContext: 'kmaster14' })
    ).toBe('kmaster14 · 10.0.0.14:6443')
  })

  it('says so when a cluster has no kubeconfig yet', () => {
    expect(endpointOf({ kind: 'KUBERNETES', baseUrl: null, kubeContext: null })).toBe('no kubeconfig yet')
  })
})
