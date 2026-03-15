import { useQuery } from '@tanstack/react-query'
import { fetchMergedPrompt } from '../api/layers'

interface Props {
  projectId: string
  variables: Record<string, string>
}

export default function MergedPreview({ projectId, variables }: Props) {
  const hasVars = Object.keys(variables).length > 0

  const { data, isLoading } = useQuery({
    queryKey: ['merged-prompt', projectId, variables],
    queryFn: () => fetchMergedPrompt(projectId, hasVars ? variables : undefined),
    refetchInterval: false,
  })

  return (
    <div className="h-full flex flex-col">
      <div className="px-3 py-2 bg-gray-50 border-b border-gray-200 text-sm font-medium text-gray-600">
        Merged Preview
      </div>
      <div className="flex-1 overflow-auto p-4">
        {isLoading ? (
          <span className="text-gray-400">Loading...</span>
        ) : (
          <pre className="whitespace-pre-wrap text-sm font-mono text-gray-800">
            {data?.merged_prompt || '(empty)'}
          </pre>
        )}
      </div>
    </div>
  )
}
