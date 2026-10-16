import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import TokenTrendChart from '@/features/tokens/components/TokenTrendChart.vue'
import type { TrendPoint } from '@/features/tokens/lib/token-trend'

const POINTS: TrendPoint[] = [
  { day: '2026-10-10', generated: 0, cumulative: 0 },
  { day: '2026-10-11', generated: 1_200, cumulative: 1_200 },
  { day: '2026-10-12', generated: 3_000, cumulative: 4_200 }
]

describe('the token trend chart', () => {
  const mountChart = () => mount(TokenTrendChart, { props: { points: POINTS, color: '#6c91da' } })
  const readout = (wrapper: ReturnType<typeof mountChart>): string => wrapper.get('[data-testid="token-trend-readout"]').text()

  it('reads out the latest day until a day is chosen', () => {
    expect(readout(mountChart())).toBe('Oct 12, 4.2k +3k')
  })

  it('moves along the days with the arrow keys, and to the ends with Home and End', async () => {
    const wrapper = mountChart()
    const chart = wrapper.get('[data-testid="token-trend-chart"]')

    await chart.trigger('keydown', { key: 'ArrowLeft' })
    expect(readout(wrapper)).toBe('Oct 11, 1.2k +1.2k')

    await chart.trigger('keydown', { key: 'Home' })
    expect(readout(wrapper)).toBe('Oct 10, 0')

    await chart.trigger('keydown', { key: 'ArrowLeft' })
    expect(readout(wrapper)).toBe('Oct 10, 0')

    await chart.trigger('keydown', { key: 'End' })
    expect(readout(wrapper)).toBe('Oct 12, 4.2k +3k')
  })

  it('goes back to the latest day when focus leaves', async () => {
    const wrapper = mountChart()
    const chart = wrapper.get('[data-testid="token-trend-chart"]')
    await chart.trigger('keydown', { key: 'Home' })

    await chart.trigger('blur')

    expect(readout(wrapper)).toBe('Oct 12, 4.2k +3k')
  })

  it('draws the line from the points and labels the first and last day', () => {
    const wrapper = mountChart()

    expect(wrapper.findAll('path')[1]!.attributes('d')).toMatch(/^M[\d. ]+ L/)
    expect(wrapper.get('svg').text()).toContain('Oct 10')
    expect(wrapper.get('svg').text()).toContain('Oct 12')
  })
})
