import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useNavigate, useParams } from 'react-router'
import { fetchPrompts, createPrompt } from '../api/prompts'

export default function PromptList() {
  const navigate = useNavigate()
  const { id: activeId } = useParams()
  const queryClient = useQueryClient()

  const { data: prompts = [], isLoading } = useQuery({
    queryKey: ['prompts'],
    queryFn: fetchPrompts,
  })

  const createMutation = useMutation({
    mutationFn: () => createPrompt({ name: 'New Prompt' }),
    onSuccess: (newPrompt) => {
      queryClient.invalidateQueries({ queryKey: ['prompts'] })
      navigate(`/prompts/${newPrompt.id}`)
    },
  })

  if (isLoading) return <div className="p-4 text-gray-500">Loading...</div>

  return (
    <div>
      <div className="p-2">
        <button
          onClick={() => createMutation.mutate()}
          disabled={createMutation.isPending}
          className="w-full px-3 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
        >
          + New Prompt
        </button>
      </div>
      <ul>
        {prompts.map((prompt) => (
          <li key={prompt.id}>
            <button
              onClick={() => navigate(`/prompts/${prompt.id}`)}
              className={`w-full text-left px-4 py-2 text-sm truncate hover:bg-gray-100 ${
                activeId === prompt.id ? 'bg-blue-50 text-blue-700 font-medium' : ''
              }`}
            >
              {prompt.name}
            </button>
          </li>
        ))}
      </ul>
    </div>
  )
}
