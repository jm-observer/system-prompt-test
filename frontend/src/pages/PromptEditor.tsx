import { useState, useEffect } from 'react'
import { useParams, useNavigate } from 'react-router'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { fetchPrompt, updatePrompt, deletePrompt } from '../api/prompts'

export default function PromptEditor() {
  const { id } = useParams()
  const navigate = useNavigate()
  const queryClient = useQueryClient()

  const [name, setName] = useState('')
  const [content, setContent] = useState('')

  const { data: prompt } = useQuery({
    queryKey: ['prompt', id],
    queryFn: () => fetchPrompt(id!),
    enabled: !!id,
  })

  useEffect(() => {
    if (prompt) {
      setName(prompt.name)
      setContent(prompt.content)
    }
  }, [prompt])

  const saveMutation = useMutation({
    mutationFn: () => updatePrompt(id!, { name, content }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['prompts'] })
      queryClient.invalidateQueries({ queryKey: ['prompt', id] })
    },
  })

  const deleteMutation = useMutation({
    mutationFn: () => deletePrompt(id!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['prompts'] })
      navigate('/prompts')
    },
  })

  if (!id) {
    return (
      <div className="flex items-center justify-center h-full text-gray-400">
        Select a prompt or create a new one
      </div>
    )
  }

  return (
    <div className="p-6 max-w-4xl">
      <div className="flex items-center gap-4 mb-4">
        <input
          value={name}
          onChange={(e) => setName(e.target.value)}
          className="text-xl font-semibold flex-1 border-b border-gray-300 focus:border-blue-500 focus:outline-none pb-1"
          placeholder="Prompt name"
        />
      </div>

      <textarea
        value={content}
        onChange={(e) => setContent(e.target.value)}
        className="w-full h-96 p-4 border border-gray-300 rounded-lg font-mono text-sm resize-y focus:ring-2 focus:ring-blue-500 focus:border-transparent"
        placeholder="Enter your system prompt here..."
      />

      <div className="flex gap-3 mt-4">
        <button
          onClick={() => saveMutation.mutate()}
          disabled={saveMutation.isPending}
          className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
        >
          {saveMutation.isPending ? 'Saving...' : 'Save'}
        </button>
        <button
          onClick={() => {
            if (confirm('Delete this prompt?')) deleteMutation.mutate()
          }}
          disabled={deleteMutation.isPending}
          className="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700 disabled:opacity-50"
        >
          Delete
        </button>
      </div>
    </div>
  )
}
