import { useState, useEffect, useCallback } from 'react'
import { useParams, useNavigate } from 'react-router'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { fetchProject, updateProject, deleteProject } from '../api/projects'
import { fetchLayers, updateLayer } from '../api/layers'
import type { PromptLayer } from '../api/layers'
import LayerTabs from '../components/LayerTabs'
import MonacoEditorWrapper from '../components/MonacoEditorWrapper'
import MergedPreview from '../components/MergedPreview'
import VariablePanel from '../components/VariablePanel'
import VersionHistory from '../components/VersionHistory'
import TestCasePanel from '../components/TestCasePanel'
import RunPanel from '../components/RunPanel'
import RunResultsView from '../components/RunResultsView'
import type { Run } from '../api/testCases'

export default function ProjectEditor() {
  const { id } = useParams()
  const navigate = useNavigate()
  const queryClient = useQueryClient()

  const [name, setName] = useState('')
  const [activeLayer, setActiveLayer] = useState<PromptLayer | null>(null)
  const [editorContent, setEditorContent] = useState('')
  const [variables, setVariables] = useState<Record<string, string>>({})
  const [selectedCaseId, setSelectedCaseId] = useState<string | null>(null)
  const [activeRuns, setActiveRuns] = useState<Run[]>([])
  const [activeTab, setActiveTab] = useState<'editor' | 'test'>('editor')

  const { data: project } = useQuery({
    queryKey: ['project', id],
    queryFn: () => fetchProject(id!),
    enabled: !!id,
  })

  const { data: layers = [] } = useQuery({
    queryKey: ['layers', id],
    queryFn: () => fetchLayers(id!),
    enabled: !!id,
  })

  useEffect(() => {
    if (project) setName(project.name)
  }, [project])

  useEffect(() => {
    if (layers.length > 0 && !activeLayer) {
      setActiveLayer(layers[0])
      setEditorContent(layers[0].content)
    }
  }, [layers, activeLayer])

  const handleLayerSelect = useCallback((layer: PromptLayer) => {
    setActiveLayer(layer)
    setEditorContent(layer.content)
  }, [])

  const saveMutation = useMutation({
    mutationFn: async () => {
      if (id) await updateProject(id, { name })
      if (activeLayer) await updateLayer(activeLayer.id, { content: editorContent })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['projects'] })
      queryClient.invalidateQueries({ queryKey: ['project', id] })
      queryClient.invalidateQueries({ queryKey: ['layers', id] })
      queryClient.invalidateQueries({ queryKey: ['merged-prompt'] })
      queryClient.invalidateQueries({ queryKey: ['versions'] })
    },
  })

  const deleteMutation = useMutation({
    mutationFn: () => deleteProject(id!),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['projects'] })
      navigate('/projects')
    },
  })

  if (!id) {
    return (
      <div className="flex items-center justify-center h-full text-gray-400">
        Select a project or create a new one
      </div>
    )
  }

  // Combine all layer contents for variable extraction
  const allContent = layers.map((l) => l.content).join('\n')

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center gap-4 px-6 py-3 border-b border-gray-200">
        <input
          value={name}
          onChange={(e) => setName(e.target.value)}
          className="text-xl font-semibold flex-1 border-b border-transparent focus:border-blue-500 focus:outline-none pb-1"
          placeholder="Project name"
        />
        <div className="flex gap-1 bg-gray-100 rounded p-0.5">
          <button
            onClick={() => setActiveTab('editor')}
            className={`px-3 py-1 text-sm rounded ${activeTab === 'editor' ? 'bg-white shadow' : ''}`}
          >
            Editor
          </button>
          <button
            onClick={() => setActiveTab('test')}
            className={`px-3 py-1 text-sm rounded ${activeTab === 'test' ? 'bg-white shadow' : ''}`}
          >
            Test
          </button>
        </div>
        <button
          onClick={() => saveMutation.mutate()}
          disabled={saveMutation.isPending}
          className="px-4 py-1.5 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
        >
          {saveMutation.isPending ? 'Saving...' : 'Save'}
        </button>
        <button
          onClick={() => {
            if (confirm('Delete this project?')) deleteMutation.mutate()
          }}
          className="px-4 py-1.5 text-sm bg-red-600 text-white rounded hover:bg-red-700"
        >
          Delete
        </button>
      </div>

      <div className="flex-1 overflow-hidden flex flex-col">
        {activeTab === 'editor' ? (
          <>
            {/* Layer Tabs */}
            <LayerTabs
              layers={layers}
              activeLayerId={activeLayer?.id ?? null}
              onSelect={handleLayerSelect}
            />

            {/* Editor + Preview */}
            <div className="flex flex-1 min-h-0">
              <div className="flex-1 border-r border-gray-200">
                <MonacoEditorWrapper
                  value={editorContent}
                  onChange={setEditorContent}
                  height="100%"
                />
              </div>
              <div className="w-96 bg-gray-50 flex flex-col">
                <div className="flex-1 overflow-y-auto">
                  <MergedPreview projectId={id} variables={variables} />
                </div>
                <VersionHistory layerId={activeLayer?.id ?? ''} />
              </div>
            </div>
          </>
        ) : (
          <div className="flex-1 overflow-y-auto p-6 space-y-4">
            <TestCasePanel
              projectId={id}
              selectedCaseId={selectedCaseId}
              onSelect={setSelectedCaseId}
            />

            {selectedCaseId && (
              <>
                <RunPanel
                  testCaseId={selectedCaseId}
                  variables={variables}
                  onRunsCreated={setActiveRuns}
                />
                <RunResultsView
                  testCaseId={selectedCaseId}
                  activeRuns={activeRuns}
                />
              </>
            )}
          </div>
        )}

        {/* Variable Panel - Always shown at bottom if variables exist */}
        <div className="border-t border-gray-200">
          <VariablePanel
            content={allContent}
            variables={variables}
            onChange={setVariables}
          />
        </div>
      </div>
    </div>
  )
}
