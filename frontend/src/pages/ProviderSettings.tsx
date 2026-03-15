import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import {
  fetchProviders, createProvider, updateProvider, deleteProvider,
  fetchModels, createModel, deleteModel,
  type ProviderResponse, type AiModel,
} from '../api/providers'

export default function ProviderSettings() {
  const qc = useQueryClient()
  const { data: providers = [] } = useQuery({ queryKey: ['providers'], queryFn: fetchProviders })

  const [showForm, setShowForm] = useState(false)
  const [editId, setEditId] = useState<string | null>(null)
  const [form, setForm] = useState({ name: '', api_type: 'openai', base_url: 'https://api.openai.com', api_key: '' })

  const createMut = useMutation({
    mutationFn: () => createProvider(form),
    onSuccess: () => { qc.invalidateQueries({ queryKey: ['providers'] }); resetForm() },
  })

  const updateMut = useMutation({
    mutationFn: () => updateProvider(editId!, { name: form.name, base_url: form.base_url, api_key: form.api_key || undefined }),
    onSuccess: () => { qc.invalidateQueries({ queryKey: ['providers'] }); resetForm() },
  })

  const deleteMut = useMutation({
    mutationFn: deleteProvider,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['providers'] }),
  })

  function resetForm() {
    setShowForm(false)
    setEditId(null)
    setForm({ name: '', api_type: 'openai', base_url: 'https://api.openai.com', api_key: '' })
  }

  function startEdit(p: ProviderResponse) {
    setEditId(p.id)
    setShowForm(true)
    setForm({ name: p.name, api_type: p.api_type, base_url: p.base_url, api_key: '' })
  }

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <div className="flex justify-between items-center mb-6">
        <h2 className="text-xl font-bold">Provider Settings</h2>
        <button
          onClick={() => { resetForm(); setShowForm(true) }}
          className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 text-sm"
        >
          Add Provider
        </button>
      </div>

      {showForm && (
        <div className="border rounded p-4 mb-6 bg-gray-50">
          <h3 className="font-semibold mb-3">{editId ? 'Edit Provider' : 'New Provider'}</h3>
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="block text-sm text-gray-600 mb-1">Name</label>
              <input
                value={form.name}
                onChange={e => setForm(f => ({ ...f, name: e.target.value }))}
                className="w-full border rounded px-3 py-2 text-sm"
                placeholder="My OpenAI"
              />
            </div>
            <div>
              <label className="block text-sm text-gray-600 mb-1">API Type</label>
              <select
                value={form.api_type}
                onChange={e => setForm(f => ({ ...f, api_type: e.target.value }))}
                className="w-full border rounded px-3 py-2 text-sm"
                disabled={!!editId}
              >
                <option value="openai">OpenAI</option>
                <option value="anthropic">Anthropic</option>
              </select>
            </div>
            <div>
              <label className="block text-sm text-gray-600 mb-1">Base URL</label>
              <input
                value={form.base_url}
                onChange={e => setForm(f => ({ ...f, base_url: e.target.value }))}
                className="w-full border rounded px-3 py-2 text-sm"
              />
            </div>
            <div>
              <label className="block text-sm text-gray-600 mb-1">API Key {editId && '(leave blank to keep)'}</label>
              <input
                value={form.api_key}
                onChange={e => setForm(f => ({ ...f, api_key: e.target.value }))}
                type="password"
                className="w-full border rounded px-3 py-2 text-sm"
                placeholder={editId ? '********' : 'sk-...'}
              />
            </div>
          </div>
          <div className="flex gap-2 mt-3">
            <button
              onClick={() => editId ? updateMut.mutate() : createMut.mutate()}
              className="px-4 py-2 bg-blue-600 text-white rounded text-sm hover:bg-blue-700"
            >
              {editId ? 'Update' : 'Create'}
            </button>
            <button onClick={resetForm} className="px-4 py-2 border rounded text-sm">
              Cancel
            </button>
          </div>
        </div>
      )}

      <div className="space-y-4">
        {providers.map(p => (
          <ProviderCard
            key={p.id}
            provider={p}
            onEdit={() => startEdit(p)}
            onDelete={() => deleteMut.mutate(p.id)}
          />
        ))}
        {providers.length === 0 && (
          <p className="text-gray-400 text-center py-8">No providers configured yet.</p>
        )}
      </div>
    </div>
  )
}

function ProviderCard({ provider, onEdit, onDelete }: {
  provider: ProviderResponse
  onEdit: () => void
  onDelete: () => void
}) {
  const qc = useQueryClient()
  const { data: models = [] } = useQuery({
    queryKey: ['models', provider.id],
    queryFn: () => fetchModels(provider.id),
  })

  const [newModel, setNewModel] = useState('')

  const addModelMut = useMutation({
    mutationFn: () => createModel(provider.id, { model_name: newModel }),
    onSuccess: () => { qc.invalidateQueries({ queryKey: ['models', provider.id] }); setNewModel('') },
  })

  const deleteModelMut = useMutation({
    mutationFn: deleteModel,
    onSuccess: () => qc.invalidateQueries({ queryKey: ['models', provider.id] }),
  })

  return (
    <div className="border rounded p-4">
      <div className="flex justify-between items-start">
        <div>
          <h3 className="font-semibold">{provider.name}</h3>
          <p className="text-sm text-gray-500">
            {provider.api_type} &middot; {provider.base_url} &middot; Key: {provider.api_key_masked}
          </p>
        </div>
        <div className="flex gap-2">
          <button onClick={onEdit} className="text-sm text-blue-600 hover:underline">Edit</button>
          <button onClick={onDelete} className="text-sm text-red-600 hover:underline">Delete</button>
        </div>
      </div>

      <div className="mt-3">
        <h4 className="text-sm font-medium text-gray-700 mb-2">Models</h4>
        <div className="flex flex-wrap gap-2 mb-2">
          {models.map((m: AiModel) => (
            <span key={m.id} className="inline-flex items-center gap-1 px-2 py-1 bg-gray-100 rounded text-sm">
              {m.model_name}
              <button
                onClick={() => deleteModelMut.mutate(m.id)}
                className="text-gray-400 hover:text-red-500"
              >
                x
              </button>
            </span>
          ))}
        </div>
        <div className="flex gap-2">
          <input
            value={newModel}
            onChange={e => setNewModel(e.target.value)}
            placeholder="e.g. gpt-4o"
            className="border rounded px-2 py-1 text-sm flex-1"
            onKeyDown={e => e.key === 'Enter' && newModel && addModelMut.mutate()}
          />
          <button
            onClick={() => newModel && addModelMut.mutate()}
            className="px-3 py-1 bg-gray-200 rounded text-sm hover:bg-gray-300"
          >
            Add
          </button>
        </div>
      </div>
    </div>
  )
}
