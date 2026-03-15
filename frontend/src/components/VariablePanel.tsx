import { useMemo } from 'react'

interface Props {
  content: string
  variables: Record<string, string>
  onChange: (variables: Record<string, string>) => void
}

export default function VariablePanel({ content, variables, onChange }: Props) {
  const varNames = useMemo(() => {
    const matches = content.match(/\{\{(\w+)\}\}/g) || []
    const names = matches.map((m) => m.slice(2, -2))
    return [...new Set(names)]
  }, [content])

  if (varNames.length === 0) return null

  return (
    <div className="border-t border-gray-200 px-4 py-3">
      <div className="text-sm font-medium text-gray-600 mb-2">Variables</div>
      <div className="flex flex-wrap gap-3">
        {varNames.map((name) => (
          <label key={name} className="flex items-center gap-2 text-sm">
            <span className="font-mono text-gray-500">{`{{${name}}}`}</span>
            <input
              type="text"
              value={variables[name] ?? ''}
              onChange={(e) => onChange({ ...variables, [name]: e.target.value })}
              className="border border-gray-300 rounded px-2 py-1 text-sm w-40 focus:ring-1 focus:ring-blue-500 focus:border-blue-500"
              placeholder={name}
            />
          </label>
        ))}
      </div>
    </div>
  )
}
