import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { fetchVersions, rollbackVersion, fetchDiff } from '../api/layers'
import DiffViewer from './DiffViewer'

interface Props {
  layerId: string
}

export default function VersionHistory({ layerId }: Props) {
  const queryClient = useQueryClient()
  const [diffV1, setDiffV1] = useState<number | null>(null)
  const [diffV2, setDiffV2] = useState<number | null>(null)
  const [showDiff, setShowDiff] = useState(false)

  const { data: versions = [] } = useQuery({
    queryKey: ['versions', layerId],
    queryFn: () => fetchVersions(layerId),
  })

  const rollbackMutation = useMutation({
    mutationFn: (version: number) => rollbackVersion(layerId, version),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['versions', layerId] })
      queryClient.invalidateQueries({ queryKey: ['layers'] })
      queryClient.invalidateQueries({ queryKey: ['merged-prompt'] })
    },
  })

  const { data: diffResult } = useQuery({
    queryKey: ['diff', layerId, diffV1, diffV2],
    queryFn: () => fetchDiff(layerId, diffV1!, diffV2!),
    enabled: showDiff && diffV1 != null && diffV2 != null,
  })

  if (versions.length === 0) return null

  return (
    <div className="border-t border-gray-200">
      <div className="px-4 py-2 flex items-center gap-3">
        <span className="text-sm font-medium text-gray-600">Versions</span>
        <div className="flex gap-1 flex-wrap">
          {versions.map((v) => (
            <button
              key={v.version}
              onClick={() => {
                if (diffV1 == null) setDiffV1(v.version)
                else if (diffV2 == null) {
                  setDiffV2(v.version)
                  setShowDiff(true)
                } else {
                  setDiffV1(v.version)
                  setDiffV2(null)
                  setShowDiff(false)
                }
              }}
              className={`px-2 py-1 text-xs rounded border ${
                diffV1 === v.version || diffV2 === v.version
                  ? 'bg-blue-100 border-blue-400 text-blue-700'
                  : 'border-gray-300 text-gray-600 hover:bg-gray-50'
              }`}
            >
              v{v.version}
            </button>
          ))}
        </div>
        {diffV1 != null && diffV2 == null && (
          <span className="text-xs text-gray-400">Select second version to diff</span>
        )}
        {showDiff && (
          <button
            onClick={() => {
              setShowDiff(false)
              setDiffV1(null)
              setDiffV2(null)
            }}
            className="text-xs text-gray-400 hover:text-gray-600"
          >
            Clear
          </button>
        )}
        {versions.length > 1 && (
          <button
            onClick={() => {
              const prev = versions[1] // second newest
              if (prev && confirm(`Rollback to v${prev.version}?`)) {
                rollbackMutation.mutate(prev.version)
              }
            }}
            disabled={rollbackMutation.isPending}
            className="ml-auto text-xs px-2 py-1 bg-yellow-100 text-yellow-800 rounded hover:bg-yellow-200 disabled:opacity-50"
          >
            Rollback
          </button>
        )}
      </div>
      {showDiff && diffResult && <DiffViewer diff={diffResult} />}
    </div>
  )
}
