import { useState, useEffect, useCallback } from 'react'
import { useQuery } from '@tanstack/react-query'
import { fetchRuns, type RunWithResult, type Run } from '../api/testCases'
import { streamRun, type StreamEvent } from '../api/sse'

interface Props {
  testCaseId: string
  activeRuns: Run[]
}

export default function RunResultsView({ testCaseId, activeRuns }: Props) {
  const { data: historicalRuns = [] } = useQuery({
    queryKey: ['runs', testCaseId],
    queryFn: () => fetchRuns(testCaseId),
    refetchInterval: activeRuns.length > 0 ? 2000 : false,
  })

  // For active runs, use SSE streaming
  const [streamContent, setStreamContent] = useState<Record<string, string>>({})

  const handleStreamEvent = useCallback((runId: string, event: StreamEvent) => {
    if (event.event_type === 'delta' && event.content) {
      setStreamContent(prev => ({
        ...prev,
        [runId]: (prev[runId] || '') + event.content,
      }))
    }
  }, [])

  useEffect(() => {
    const cleanups: (() => void)[] = []
    for (const run of activeRuns) {
      const cleanup = streamRun(
        run.id,
        (event) => handleStreamEvent(run.id, event),
      )
      cleanups.push(cleanup)
    }
    return () => cleanups.forEach(fn => fn())
  }, [activeRuns, handleStreamEvent])

  // Merge active stream content with historical results
  const allRuns = historicalRuns.length > 0 ? historicalRuns : activeRuns.map(r => ({
    ...r,
    result: null,
  } as RunWithResult))

  if (allRuns.length === 0) {
    return <p className="text-sm text-gray-400 py-4">No runs yet. Select a test case and models to run.</p>
  }

  return (
    <div className="space-y-3">
      <h4 className="text-sm font-medium">Run Results</h4>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
        {allRuns.map(run => {
          const streaming = streamContent[run.id]
          const resultText = run.result?.response_text || streaming || ''
          const error = run.result?.error_message

          return (
            <div key={run.id} className="border rounded p-3">
              <div className="flex justify-between items-center mb-2">
                <span className="text-sm font-medium">{run.model_id}</span>
                <StatusBadge status={run.status} />
              </div>

              <div className="bg-gray-50 rounded p-2 text-sm font-mono whitespace-pre-wrap max-h-64 overflow-y-auto">
                {error ? (
                  <span className="text-red-600">{error}</span>
                ) : resultText ? (
                  resultText
                ) : (
                  <span className="text-gray-400">Waiting...</span>
                )}
              </div>

              {run.result && (
                <div className="mt-2 flex gap-3 text-xs text-gray-500">
                  {run.result.latency_ms && <span>{run.result.latency_ms}ms</span>}
                  <span>{run.result.token_usage}</span>
                </div>
              )}
            </div>
          )
        })}
      </div>
    </div>
  )
}

function StatusBadge({ status }: { status: string }) {
  const colors: Record<string, string> = {
    running: 'bg-yellow-100 text-yellow-700',
    completed: 'bg-green-100 text-green-700',
    failed: 'bg-red-100 text-red-700',
  }

  return (
    <span className={`px-2 py-0.5 rounded text-xs ${colors[status] || 'bg-gray-100'}`}>
      {status}
    </span>
  )
}
