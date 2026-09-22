import type { config } from 'md-editor-v3'

/** The markdown-it instance md-editor-v3 hands its config hook, typed through the hook itself so
 *  this file needs no direct dependency on markdown-it. */
type MarkdownIt = Parameters<NonNullable<Parameters<typeof config>[0]['markdownItConfig']>>[0]

/**
 * An inline code span that is nothing but anchors (`project:vega task:report-builder`), with or
 * without the `/rk` in front of it. The same four entities `rekall_context` resolves, and nothing
 * else: `http://x` or `a:b` in a code span is still just code.
 */
const ANCHOR_RUN = /^(\/rk\s+)?((company|project|task|note):[\w.-]+)(\s+(company|project|task|note):[\w.-]+)*$/

export function isAnchorRun(content: string): boolean {
  return ANCHOR_RUN.test(content.trim())
}

/**
 * Renders an anchor-only code span with the `rk-anchor` class, so a brief, a wrapup or a note
 * shows the strings a session can load in the one colour the console gives anchors, and the rest
 * of its code in a neutral one.
 */
export function markAnchorCode(md: MarkdownIt): void {
  const render = md.renderer.rules.code_inline
  md.renderer.rules.code_inline = (tokens, index, options, env, renderer) => {
    const token = tokens[index]
    if (token && isAnchorRun(token.content)) token.attrJoin('class', 'rk-anchor')
    return render
      ? render(tokens, index, options, env, renderer)
      : renderer.renderToken(tokens, index, options)
  }
}
