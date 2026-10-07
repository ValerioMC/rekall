import { afterEach, describe, expect, it } from 'vitest'
import { markdown, markdownLanguage } from '@codemirror/lang-markdown'
import { EditorState } from '@codemirror/state'
import { EditorView } from '@codemirror/view'
import { livePreview } from '@/common/live-preview/live-preview'
import { editTableCell, revealTableSource, transformTable } from '@/common/live-preview/table-edit'
import { TABLE_MENU_EVENT, type TableMenuRequest } from '@/common/live-preview/table-widget'

const TABLE = '| Piece | Role |\n|---|---|\n| Clock | tells the time |'

const views: EditorView[] = []

function editorOn(doc: string, caret = 0): EditorView {
  const view = new EditorView({
    parent: document.body.appendChild(document.createElement('div')),
    state: EditorState.create({
      doc,
      selection: { anchor: caret },
      extensions: [markdown({ base: markdownLanguage }), livePreview()]
    })
  })
  views.push(view)
  return view
}

/** CodeMirror learns of focus from the DOM event and reports it a moment later. */
async function focus(view: EditorView): Promise<void> {
  view.focus()
  view.contentDOM.dispatchEvent(new FocusEvent('focus'))
  await new Promise((resolve) => setTimeout(resolve, 30))
}

function lineText(view: EditorView, number: number): string {
  return view.contentDOM.querySelectorAll('.cm-line')[number - 1]?.textContent ?? ''
}

afterEach(() => {
  views.splice(0).forEach((view) => view.destroy())
  document.body.innerHTML = ''
})

describe('live preview', () => {
  it('reads as the page while unfocused: markup hidden, styles applied', () => {
    const view = editorOn('# Title\n\nSome **bold** and *slanted* text')

    expect(view.contentDOM.textContent).not.toContain('#')
    expect(view.contentDOM.textContent).not.toContain('*')
    expect(view.contentDOM.querySelector('.cm-rk-h1')?.textContent).toBe('Title')
    expect(view.contentDOM.querySelector('.cm-rk-strong')?.textContent).toBe('bold')
    expect(view.contentDOM.querySelector('.cm-rk-em')?.textContent).toBe('slanted')
  })

  it('shows the markup of the line holding the caret, and only that line', async () => {
    const doc = '# Title\n\nSome **bold** text'
    const view = editorOn(doc, doc.indexOf('bold'))
    await focus(view)

    expect(lineText(view, 3)).toBe('Some **bold** text')
    expect(lineText(view, 1)).toBe('Title')
  })

  it('never changes the text it draws', async () => {
    const doc = `# Title\n\n- [ ] open\n- item\n\n${TABLE}\n\n\`\`\`js\nconst a = 1\n\`\`\`\n`
    const view = editorOn(doc)
    await focus(view)
    view.dispatch({ selection: { anchor: doc.indexOf('item') } })

    expect(view.state.doc.toString()).toBe(doc)
  })

  it('colours an anchor-only code span as an anchor and other code neutrally', () => {
    const view = editorOn('Load `project:vega task:report` with `kubectl`.')
    const spans = [...view.contentDOM.querySelectorAll('.cm-rk-code')]

    expect(spans.map((span) => span.textContent)).toEqual(['project:vega task:report', 'kubectl'])
    expect(spans.map((span) => span.classList.contains('rk-anchor'))).toEqual([true, false])
  })

  it('shows a link by its text alone', () => {
    const view = editorOn('See [the docs](https://example.com/docs) first.')

    expect(view.contentDOM.textContent).toBe('See the docs first.')
    expect(view.contentDOM.querySelector('.cm-rk-link')?.textContent).toBe('the docs')
  })

  it('draws a bullet as the read view dash and a task as a checkbox', () => {
    const view = editorOn('- item\n- [ ] open\n- [x] shut')

    expect(view.contentDOM.querySelectorAll('.cm-rk-bullet')).toHaveLength(1)
    const boxes = [...view.contentDOM.querySelectorAll<HTMLInputElement>('input.cm-rk-task')]
    expect(boxes.map((box) => box.checked)).toEqual([false, true])
  })

  it('ticks a task by writing x into its brackets', () => {
    const view = editorOn('- [ ] ship it')

    view.contentDOM
      .querySelector('input.cm-rk-task')
      ?.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }))

    expect(view.state.doc.toString()).toBe('- [x] ship it')
  })

  it('draws a fenced block highlighted, and its source once the caret is inside', async () => {
    const doc = 'Intro\n\n```js\nconst answer = 42\n```'
    const view = editorOn(doc)

    expect(view.dom.querySelector('.cm-rk-codeblock code.hljs')?.textContent).toBe('const answer = 42')
    expect(view.contentDOM.textContent).not.toContain('```')

    await focus(view)
    view.dispatch({ selection: { anchor: doc.indexOf('answer') } })

    expect(view.dom.querySelector('.cm-rk-codeblock')).toBeNull()
    expect(view.contentDOM.textContent).toContain('```js')
  })

  it('draws a table, and its markdown once the caret is inside', async () => {
    const doc = `Intro\n\n${TABLE}`
    const view = editorOn(doc)

    expect(view.dom.querySelectorAll('.cm-rk-table th')).toHaveLength(2)
    expect(view.dom.querySelectorAll('.cm-rk-table td')).toHaveLength(2)

    await focus(view)
    view.dispatch({ selection: { anchor: doc.indexOf('Clock') } })

    expect(view.dom.querySelector('.cm-rk-table')).toBeNull()
    expect(view.contentDOM.textContent).toContain('|---|---|')
  })
})

describe('table editing', () => {
  it('writes a cell edit into that cell and nowhere else', () => {
    const view = editorOn(TABLE)

    editTableCell(view, 0, { row: 1, column: 1 }, 'reads the clock')

    expect(view.state.doc.toString()).toBe(TABLE.replace('tells the time', 'reads the clock'))
  })

  it('finds the table after the text in front of it has grown', () => {
    const doc = `Intro\n\n${TABLE}`
    const view = editorOn(doc)
    view.dispatch({ changes: { from: 0, insert: 'A longer ' } })

    editTableCell(view, doc.indexOf('|') + 'A longer '.length, { row: 0, column: 0 }, 'Name')

    expect(view.state.doc.toString()).toContain('| Name | Role |')
  })

  it('adds a row by rewriting only that table', () => {
    const doc = `Before\n\n${TABLE}\n\nAfter`
    const view = editorOn(doc)

    transformTable(view, doc.indexOf('|'), { kind: 'insertRow', row: 1 })

    expect(view.state.doc.toString()).toBe(
      'Before\n\n| Piece | Role |\n| --- | --- |\n| Clock | tells the time |\n|  |  |\n\nAfter'
    )
  })

  it('opens the markdown by putting the caret at the start of the table', () => {
    const doc = `Intro\n\n${TABLE}`
    const view = editorOn(doc)

    revealTableSource(view, doc.indexOf('|'))

    expect(view.state.selection.main.head).toBe(doc.indexOf('|'))
  })

  it('edits a cell in place: its markdown while writing, written back on Enter', () => {
    const view = editorOn(TABLE)
    const cell = view.dom.querySelector<HTMLElement>('.cm-rk-table td[data-column="1"]')!

    cell.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    expect(cell.dataset.editing).toBe('true')
    expect(cell.textContent).toBe('tells the time')

    cell.textContent = 'keeps **time**'
    cell.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }))

    expect(view.state.doc.toString()).toBe(TABLE.replace('tells the time', 'keeps **time**'))
    expect(view.dom.querySelector('.cm-rk-table td[data-column="1"] strong')?.textContent).toBe('time')
  })

  it('puts a cell back untouched on Escape', () => {
    const view = editorOn(TABLE)
    const cell = view.dom.querySelector<HTMLElement>('.cm-rk-table td[data-column="0"]')!

    cell.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    cell.textContent = 'scrapped'
    cell.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))

    expect(view.state.doc.toString()).toBe(TABLE)
    expect(cell.textContent).toBe('Clock')
  })

  it('asks for the table menu on a right-click, naming the cell', () => {
    const view = editorOn(TABLE)
    const requests: TableMenuRequest[] = []
    view.dom.addEventListener(TABLE_MENU_EVENT, (event) =>
      requests.push((event as CustomEvent<TableMenuRequest>).detail)
    )

    view.dom
      .querySelector('.cm-rk-table td[data-column="1"]')
      ?.dispatchEvent(
        new MouseEvent('contextmenu', { bubbles: true, cancelable: true, clientX: 40, clientY: 50 })
      )

    expect(requests).toEqual([
      { x: 40, y: 50, tablePosition: 0, cell: { row: 1, column: 1 }, rowCount: 2, columnCount: 2 }
    ])
  })
})
