import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import {
  fetchTestCases, createTestCase, updateTestCase, deleteTestCase,
  type TestCase,
} from '../api/testCases'

interface Props {
  projectId: string
  selectedCaseId: string | null
  onSelect: (id: string | null) => void
}

export default function TestCasePanel({ projectId, selectedCaseId, onSelect }: Props) {
  const qc = useQueryClient()
  const { data: cases = [] } = useQuery({
    queryKey: ['testCases', projectId],
    queryFn: () => fetchTestCases(projectId),
  })

  const [editing, setEditing] = useState<string | null>(null)
  const [form, setForm] = useState({ name: '', user_message: '' })

  const createMut = useMutation({
    mutationFn: () => createTestCase(projectId, form),
    onSuccess: (tc) => {
      qc.invalidateQueries({ queryKey: ['testCases', projectId] })
      onSelect(tc.id)
      setForm({ name: '', user_message: '' })
    },
  })

  const updateMut = useMutation({
    mutationFn: () => updateTestCase(editing!, form),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['testCases', projectId] })
      setEditing(null)
    },
  })

  const deleteMut = useMutation({
    mutationFn: deleteTestCase,
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['testCases', projectId] })
      if (selectedCaseId === editing) onSelect(null)
    },
  })

  function startEdit(tc: TestCase) {
    setEditing(tc.id)
    setForm({ name: tc.name, user_message: tc.user_message })
  }

  function startNew() {
    setEditing('new')
    setForm({ name: '', user_message: '' })
  }

  return (
    <div className="border-t mt-4 pt-4">
      <div className="flex justify-between items-center mb-3">
        <h3 className="font-semibold text-sm">Test Cases</h3>
        <button onClick={startNew} className="text-sm text-blue-600 hover:underline">
          + New
        </button>
      </div>

      {editing && (
        <div className="border rounded p-3 mb-3 bg-gray-50">
          <input
            value={form.name}
            onChange={e => setForm(f => ({ ...f, name: e.target.value }))}
            placeholder="Test case name"
            className="w-full border rounded px-2 py-1 text-sm mb-2"
          />
          <textarea
            value={form.user_message}
            onChange={e => setForm(f => ({ ...f, user_message: e.target.value }))}
            placeholder="User message..."
            className="w-full border rounded px-2 py-1 text-sm h-20 resize-none"
          />
          <div className="flex gap-2 mt-2">
            <button
              onClick={() => editing === 'new' ? createMut.mutate() : updateMut.mutate()}
              className="px-3 py-1 bg-blue-600 text-white rounded text-sm"
            >
              {editing === 'new' ? 'Create' : 'Save'}
            </button>
            <button onClick={() => setEditing(null)} className="px-3 py-1 border rounded text-sm">
              Cancel
            </button>
          </div>
        </div>
      )}

      <div className="space-y-1">
        {cases.map(tc => (
          <div
            key={tc.id}
            onClick={() => onSelect(tc.id)}
            className={`flex justify-between items-center px-3 py-2 rounded cursor-pointer text-sm ${
              selectedCaseId === tc.id ? 'bg-blue-50 border-blue-200 border' : 'hover:bg-gray-50'
            }`}
          >
            <span>{tc.name}</span>
            <div className="flex gap-2">
              <button
                onClick={e => { e.stopPropagation(); startEdit(tc) }}
                className="text-gray-400 hover:text-blue-600"
              >
                Edit
              </button>
              <button
                onClick={e => { e.stopPropagation(); deleteMut.mutate(tc.id) }}
                className="text-gray-400 hover:text-red-600"
              >
                Del
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
