import { describe, expect, it } from 'vitest'
import { nextTick, ref } from 'vue'
import { useSettledOrder } from '@/composables/useSettledOrder'

type Row = Readonly<{ id: string; attached: boolean }>

const attachedFirst = (rows: readonly Row[]): Row[] => [
  ...rows.filter((row) => row.attached),
  ...rows.filter((row) => !row.attached)
]

describe('a settled picker order', () => {
  it('keeps a ticked row where it was instead of moving it to the top', () => {
    const rows = ref<Row[]>([
      { id: 'a', attached: true },
      { id: 'b', attached: false },
      { id: 'c', attached: false }
    ])
    const settled = useSettledOrder(() => attachedFirst(rows.value), (row) => row.id)
    expect(settled.value.map((row) => row.id)).toEqual(['a', 'b', 'c'])

    rows.value = rows.value.map((row) => (row.id === 'c' ? { ...row, attached: true } : row))
    expect(settled.value.map((row) => row.id)).toEqual(['a', 'b', 'c'])
  })

  it('puts a row it has not seen before at the end', () => {
    const rows = ref<Row[]>([{ id: 'b', attached: false }])
    const settled = useSettledOrder(() => rows.value, (row) => row.id)
    void settled.value

    rows.value = [{ id: 'a', attached: false }, ...rows.value]
    expect(settled.value.map((row) => row.id)).toEqual(['b', 'a'])
  })

  it('lets the source order count again once the filter changes', async () => {
    const filter = ref('')
    const rows = ref<Row[]>([
      { id: 'a', attached: false },
      { id: 'b', attached: false }
    ])
    const settled = useSettledOrder(() => attachedFirst(rows.value), (row) => row.id, [filter])
    void settled.value

    rows.value = [rows.value[0]!, { id: 'b', attached: true }]
    expect(settled.value.map((row) => row.id)).toEqual(['a', 'b'])

    filter.value = 'x'
    await nextTick()
    expect(settled.value.map((row) => row.id)).toEqual(['b', 'a'])
  })
})
