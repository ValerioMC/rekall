import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import DecisionRow from '@/features/decisions/components/DecisionRow.vue'
import { searchDecisions } from '@/features/decisions/lib/decision-search'
import type { Decision } from '@/features/decisions/model/decision'
import { answered, decision, onHold } from './fixtures'

function row(entry: Decision, props: { hasSteps?: boolean; opened?: boolean } = {}) {
  return mount(DecisionRow, { props: { decision: entry, hasSteps: props.hasSteps ?? false, opened: props.opened ?? false } })
}

function searchedRow(entry: Decision, query: string, opened = false) {
  const find = searchDecisions([entry], query)?.get(entry.id)
  return mount(DecisionRow, { props: { decision: entry, hasSteps: false, opened, query, find } })
}

const marked = (page: ReturnType<typeof row>) => page.findAll('mark').map((mark) => mark.text())

describe('a decision row', () => {
  it('underlines every occurrence of a searched string, as the text writes it', () => {
    const page = searchedRow(answered('formats', 'PDF', { question: 'Is the pdf offered?' }), 'pdf')

    expect(marked(page)).toEqual(['pdf', 'PDF'])
    expect(page.get('mark').classes()).toContain('underline')
    expect(page.find('[data-testid="decision-found"]').exists()).toBe(false)
  })

  it('quotes the first hidden place when the closed row shows no match, and names the others', () => {
    const entry = answered('file', 'Downloads', {
      why: 'Never overwrite.',
      lines: [{ speaker: 'CLAUDE', text: 'Do we overwrite it?', spokenAt: '2026-11-23T14:34:00Z' }]
    })
    const page = searchedRow(entry, 'overwrite')

    expect(page.get('[data-testid="decision-found"]').text()).toContain('Conversation')
    expect(page.get('[data-testid="decision-excerpt"]').text()).toBe('Do we overwrite it?')
    expect(page.get('[data-testid="decision-found"]').text()).toContain('and in why asked')
    expect(marked(page)).toEqual(['overwrite'])
  })

  it('only names the hidden places when the closed row already shows a match', () => {
    const page = searchedRow(answered('formats', 'PDF', { why: 'The PDF is read most.' }), 'pdf')

    expect(page.get('[data-testid="decision-found"]').text()).toBe('Also in why asked')
  })

  it('drops the line when opened and underlines the matches where they are', () => {
    const page = searchedRow(answered('file', 'Downloads', { why: 'Never overwrite.' }), 'overwrite', true)

    expect(page.find('[data-testid="decision-found"]').exists()).toBe(false)
    expect(page.get('[data-testid="decision-drawer"] mark').text()).toBe('overwrite')
  })

  it('keeps quoting a note the opened row cannot show', () => {
    const entry = onHold('file', 'Waiting', { details: [{ text: 'Check overwrite', addedAt: '2026-11-23T14:34:00Z' }] })
    const page = searchedRow(entry, 'overwrite', true)

    expect(page.get('[data-testid="decision-excerpt"]').text()).toBe('Check overwrite')
  })

  it('tells where an open question is being discussed and what was offered, the recommended option in bold', () => {
    const page = row(decision('db-password', { options: [{ label: 'Secret', effect: null, recommended: false }, { label: 'Vault', effect: null, recommended: true }] }))

    expect(page.text()).toContain('Still being discussed in the terminal')
    expect(page.get('[data-testid="decision-offered"]').text()).toBe('Offered: Secret · Vault')
    expect(page.get('[data-testid="decision-offered"] b').text()).toBe('Vault')
    expect(page.get('[data-testid="decision-seal"]').attributes('aria-label')).toBe('Open')
  })

  it('shows the answer, what it replaced and the dated details of an answered decision', () => {
    const page = row(
      answered('replicas', '3', {
        previousAnswer: '2',
        changedAt: '2026-11-23T14:31:00Z',
        details: [{ text: 'Autoscale to 6.', addedAt: '2026-11-23T14:31:00Z' }]
      })
    )

    expect(page.get('[data-testid="decision-answer"]').text()).toBe('3')
    expect(page.get('[data-testid="decision-was"]').text()).toContain('was 2')
    expect(page.get('[data-testid="decision-detail"]').text()).toContain('Autoscale to 6.')
    expect(page.get('[data-testid="decision-seal"]').attributes('aria-label')).toBe('Answered')
  })

  it('shows the note of a decision on hold', () => {
    const page = row(onHold('vault-path', 'Asking the platform team'))

    expect(page.get('[data-testid="decision-hold"]').text()).toBe('On hold: Asking the platform team')
    expect(page.get('[data-testid="decision-seal"]').attributes('aria-label')).toBe('On hold')
  })

  it('carries a Task or Step tag only when the task has steps', () => {
    expect(row(decision('a'), { hasSteps: false }).find('[data-testid="decision-scope"]').exists()).toBe(false)
    expect(row(decision('a'), { hasSteps: true }).get('[data-testid="decision-scope"]').text()).toBe('Task')
    expect(row(decision('a', { stepNumber: 3 }), { hasSteps: true }).get('[data-testid="decision-scope"]').text()).toBe('Step 3')
  })

  it('marks a record the person wrote and says who asked', () => {
    const page = row(answered('namespace', 'billing', { source: 'PERSON' }))

    expect(page.get('[data-testid="decision-source"]').text()).toBe('Edited by you')
    expect(page.text()).toContain('You wrote this')
  })

  it('tells the toggle state to assistive tech and asks its parent to open it', async () => {
    const page = row(decision('a'))

    expect(page.get('[data-testid="decision-toggle"]').attributes('aria-expanded')).toBe('false')
    await page.get('[data-testid="decision-toggle"]').trigger('click')

    expect(page.emitted('toggle')).toHaveLength(1)
  })

  it('opens onto the conversation, the options with the chosen one marked, and why it was asked', () => {
    const page = row(
      answered('db-password', 'Vault', {
        why: 'The plan changes with the answer.',
        stepNumber: 3,
        stepTitle: 'Write the manifests',
        options: [
          { label: 'Secret', effect: 'One manifest.', recommended: false },
          { label: 'Vault', effect: 'Rotation.', recommended: true }
        ],
        lines: [
          { speaker: 'CLAUDE', text: 'Which store?', spokenAt: '2026-11-23T14:34:00Z' },
          { speaker: 'PERSON', text: 'Vault', spokenAt: '2026-11-23T14:35:00Z' }
        ]
      }),
      { opened: true, hasSteps: true }
    )

    expect(page.get('[data-testid="decision-toggle"]').attributes('aria-expanded')).toBe('true')
    expect(page.findAll('[data-testid="decision-line"]').map((line) => line.text())).toEqual([
      expect.stringContaining('Which store?'),
      expect.stringContaining('Vault')
    ])
    const options = page.findAll('[data-testid="decision-option"]')
    expect(options[1]!.text()).toContain('Recommended')
    expect(options[1]!.text()).toContain('Your answer')
    expect(options[0]!.text()).not.toContain('Your answer')
    expect(page.get('[data-testid="decision-drawer"]').text()).toContain('The plan changes with the answer.')
    expect(page.get('[data-testid="decision-drawer"]').text()).toContain('Read when step 3, Write the manifests, runs')
  })
})
