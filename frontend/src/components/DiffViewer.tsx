import type { DiffResult } from '../api/layers'

interface Props {
  diff: DiffResult
}

export default function DiffViewer({ diff }: Props) {
  return (
    <div className="border-t border-gray-200 px-4 py-3 max-h-64 overflow-auto">
      <div className="text-xs text-gray-500 mb-2">
        v{diff.v1} → v{diff.v2}
      </div>
      <pre className="text-sm font-mono">
        {diff.changes.map((c, i) => {
          let className = ''
          let prefix = ' '
          if (c.tag === 'insert') {
            className = 'bg-green-100 text-green-800'
            prefix = '+'
          } else if (c.tag === 'delete') {
            className = 'bg-red-100 text-red-800'
            prefix = '-'
          }
          return (
            <span key={i} className={className}>
              {prefix} {c.content}
            </span>
          )
        })}
      </pre>
    </div>
  )
}
