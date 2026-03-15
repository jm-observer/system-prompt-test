import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { fetchAllModels, type AiModel } from '../api/providers'
import { createRun, type Run } from '../api/testCases'

interface Props {
  testCaseId: string
  variables: Record<string, string>
  onRunsCreated: (runs: Run[]) => void
}

export default function RunPanel({ testCaseId, variables, onRunsCreated }: Props) {
  const qc = useQueryClient()
  const { data: models = [] } = useQuery({ queryKey: ['allModels'], queryFn: fetchAllModels })
  const [selectedModels, setSelectedModels] = useState<Set<string>>(new Set())

  const runMut = useMutation({
    mutationFn: () =>
      createRun(testCaseId, {
        model_ids: Array.from(selectedModels),
        variables,
      }),
    onSuccess: (runs) => {
      qc.invalidateQueries({ queryKey: ['runs', testCaseId] })
      onRunsCreated(runs)
    },
  })

  function toggleModel(id: string) {
    setSelectedModels(prev => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }

  return (
    <div className="border rounded p-3">
      <h4 className="text-sm font-medium mb-2">Select Models to Run</h4>

      {models.length === 0 ? (
        <p className="text-sm text-gray-400">No models configured. Go to Settings to add providers and models.</p>
      ) : (
        <div className="flex flex-wrap gap-2 mb-3">
          {models.map((m: AiModel) => (
            <label
              key={m.id}
              className={`inline-flex items-center gap-1 px-2 py-1 rounded border text-sm cursor-pointer ${
                selectedModels.has(m.id) ? 'bg-blue-50 border-blue-300' : 'hover:bg-gray-50'
              }`}
            >
              <input
                type="checkbox"
                checked={selectedModels.has(m.id)}
                onChange={() => toggleModel(m.id)}
                className="accent-blue-600"
              />
              {m.model_name}
            </label>
          ))}
        </div>
      )}

      <button
        onClick={() => runMut.mutate()}
        disabled={selectedModels.size === 0 || runMut.isPending}
        className="px-4 py-2 bg-green-600 text-white rounded text-sm hover:bg-green-700 disabled:opacity-50"
      >
        {runMut.isPending ? 'Running...' : `Run (${selectedModels.size} model${selectedModels.size !== 1 ? 's' : ''})`}
      </button>
    </div>
  )
}
