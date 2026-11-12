import { afterEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import TaskTrackerMark from '@/features/catalog/components/TaskTrackerMark.vue'

type Tracked = { trackerIssueKey: string | null; trackerIssueUrl: string | null; trackerSyncError: string | null }

const render = (task: Tracked) => mount(TaskTrackerMark, { props: { task } })

describe('the task tracker mark', () => {
  afterEach(() => {
    delete window.rekallDesktop
  })

  it('shows nothing for a task the task manager does not hold', () => {
    expect(render({ trackerIssueKey: null, trackerIssueUrl: null, trackerSyncError: null }).html()).toBe('<!--v-if-->')
  })

  it('links the issue the task manager holds', () => {
    const link = render({ trackerIssueKey: 'ENG-12', trackerIssueUrl: 'https://example.atlassian.net/browse/ENG-12', trackerSyncError: null }).get(
      '[data-testid="tracker-issue-link"]'
    )

    expect(link.attributes('href')).toBe('https://example.atlassian.net/browse/ENG-12')
    expect(link.text()).toContain('ENG-12')
  })

  it('says why the last write failed', () => {
    const mark = render({ trackerIssueKey: null, trackerIssueUrl: null, trackerSyncError: 'GitLab rejected the stored credential (HTTP 401).' })

    expect(mark.find('[data-testid="tracker-issue-link"]').exists()).toBe(false)
    expect(mark.get('[data-testid="tracker-sync-error"]').attributes('title')).toContain('HTTP 401')
  })

  it('hands the link to the desktop window, which opens it in the browser', async () => {
    const openExternalLink = vi.fn().mockResolvedValue(undefined)
    window.rekallDesktop = { pickFolder: vi.fn(), openExternalLink }
    const link = render({ trackerIssueKey: '12', trackerIssueUrl: 'https://github.com/acme/app/issues/12', trackerSyncError: null }).get('[data-testid="tracker-issue-link"]')

    await link.trigger('click')

    expect(openExternalLink).toHaveBeenCalledWith('https://github.com/acme/app/issues/12')
  })
})
