import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { fetchAssertions, createAssertion, deleteAssertion } from '../api/testCases'

interface Props {
  testCaseId: string
}

export default function AssertionPanel({ testCaseId }: Props) {
  const queryClient = useQueryClient()
  const [newType, setNewType] = useState('keyword_present')
  const [newConfig, setNewConfig] = useState('{"keyword": ""}')

  const { data: assertions = [] } = useQuery({
    queryKey: ['assertions', testCaseId],
    queryFn: () => fetchAssertions(testCaseId),
  })

  const addMutation = useMutation({
    mutationFn: () => createAssertion(testCaseId, { assertion_type: newType, config: newConfig }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['assertions', testCaseId] })
    },
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteAssertion(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['assertions', testCaseId] })
    },
  })

  return (
    <div className="border rounded p-4 bg-white shadow-sm space-y-4">
      <h3 className="text-sm font-semibold text-gray-700 flex items-center gap-2">
        <svg className="w-4 h-4 text-blue-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        Assertion Rules
      </h3>

      <div className="space-y-2">
        {assertions.map((a) => (
          <div key={a.id} className="flex items-center justify-between bg-gray-50 p-2 rounded text-sm border border-gray-100">
            <div className="flex flex-col">
              <span className="font-medium text-blue-700 uppercase text-xs">{a.assertion_type.replace('_', ' ')}</span>
              <code className="text-gray-600 mt-1">{a.config}</code>
            </div>
            <button
              onClick={() => deleteMutation.mutate(a.id)}
              className="text-gray-400 hover:text-red-500 transition-colors"
            >
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
            </button>
          </div>
        ))}
        {assertions.length === 0 && (
          <p className="text-xs text-gray-400 italic">No assertions defined yet.</p>
        )}
      </div>

      <div className="pt-2 border-t border-gray-100 grid grid-cols-1 md:grid-cols-3 gap-2">
        <select
          value={newType}
          onChange={(e) => setNewType(e.target.value)}
          className="text-xs border rounded px-2 py-1 bg-white focus:ring-1 focus:ring-blue-500 outline-none"
        >
          <option value="keyword_present">Keyword Present</option>
          <option value="keyword_absent">Keyword Absent</option>
          <option value="must_call">Must Call Tool</option>
          <option value="must_not_call">Must Not Call Tool</option>
        </select>
        <input
          value={newConfig}
          onChange={(e) => setNewConfig(e.target.value)}
          placeholder='{"keyword": "foo"}'
          className="text-xs border rounded px-2 py-1 font-mono focus:ring-1 focus:ring-blue-500 outline-none"
        />
        <button
          onClick={() => addMutation.mutate()}
          className="text-xs bg-blue-600 text-white rounded py-1 hover:bg-blue-700 font-medium transition-colors"
        >
          Add Rule
        </button>
      </div>
    </div>
  )
}
