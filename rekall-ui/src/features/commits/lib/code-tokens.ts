import type { TextSegment } from '@/features/commits/lib/word-diff'

export type CodeLanguage = 'slash' | 'hash' | 'plain'

export type CodeTone = 'plain' | 'keyword' | 'string' | 'comment'

export interface CodePart {
  readonly text: string
  readonly changed: boolean
  readonly tone: CodeTone
}

const SLASH_EXTENSIONS = new Set(['ts', 'tsx', 'js', 'jsx', 'mjs', 'vue', 'rs', 'go', 'java', 'kt', 'c', 'h', 'cpp', 'css', 'scss', 'swift'])
const HASH_EXTENSIONS = new Set(['py', 'sh', 'toml', 'yml', 'yaml', 'rb'])

const KEYWORDS =
  'const|let|var|function|return|import|from|export|default|type|interface|class|extends|implements|new|if|else|for|while|switch|case|break|continue|async|await|throw|try|catch|finally|fn|pub|impl|struct|enum|trait|use|mod|match|mut|self|def|void|null|true|false|undefined'

function tokenPattern(language: CodeLanguage): RegExp {
  const comment = language === 'hash' ? '#.*$' : '\\/\\/.*$'
  return new RegExp(`(${comment})|('(?:[^'\\\\]|\\\\.)*'|"(?:[^"\\\\]|\\\\.)*"|\`(?:[^\`\\\\]|\\\\.)*\`)|\\b(${KEYWORDS})\\b`, 'g')
}

const PATTERNS: Readonly<Record<'slash' | 'hash', RegExp>> = { slash: tokenPattern('slash'), hash: tokenPattern('hash') }

export function languageOf(path: string): CodeLanguage {
  const extension = path.slice(path.lastIndexOf('.') + 1).toLowerCase()
  if (SLASH_EXTENSIONS.has(extension)) return 'slash'
  return HASH_EXTENSIONS.has(extension) ? 'hash' : 'plain'
}

/** A line split into the pieces that are drawn differently: keywords, strings, comments, and the words an edit changed. */
export function codeParts(text: string, language: CodeLanguage, segments: readonly TextSegment[] | null): readonly CodePart[] {
  const pieces = segments ?? [{ text, changed: false }]
  return pieces.flatMap((piece) => tonedParts(piece, language))
}

function tonedParts(segment: TextSegment, language: CodeLanguage): readonly CodePart[] {
  if (language === 'plain' || segment.text === '') return [{ text: segment.text, changed: segment.changed, tone: 'plain' }]
  const parts: CodePart[] = []
  let last = 0
  for (const match of segment.text.matchAll(PATTERNS[language])) {
    const index = match.index ?? 0
    if (index > last) parts.push({ text: segment.text.slice(last, index), changed: segment.changed, tone: 'plain' })
    parts.push({ text: match[0], changed: segment.changed, tone: toneOf(match) })
    last = index + match[0].length
  }
  if (last < segment.text.length) parts.push({ text: segment.text.slice(last), changed: segment.changed, tone: 'plain' })
  return parts
}

function toneOf(match: RegExpMatchArray): CodeTone {
  if (match[1] !== undefined) return 'comment'
  return match[2] !== undefined ? 'string' : 'keyword'
}
