export interface PromptLayer {
  id: string
  project_id: string
  layer_type: 'global' | 'project' | 'provider' | 'model'
  target_ref: string
  content: string
  created_at: string
  updated_at: string
}

export interface PromptVersion {
  id: string
  layer_id: string
  version: number
  content: string
  created_at: string
}

export interface DiffChange {
  tag: 'equal' | 'insert' | 'delete'
  content: string
}

export interface DiffResult {
  v1: number
  v2: number
  changes: DiffChange[]
}

export interface MergedPromptResponse {
  merged_prompt: string
  layers: { layer_type: string; has_content: boolean }[]
}

async function handleResponse<T>(res: Response): Promise<T> {
  if (!res.ok) {
    const text = await res.text()
    throw new Error(`API error ${res.status}: ${text}`)
  }
  if (res.status === 204) return undefined as T
  return res.json()
}

export async function fetchLayers(projectId: string): Promise<PromptLayer[]> {
  const res = await fetch(`/api/projects/${projectId}/layers`)
  return handleResponse<PromptLayer[]>(res)
}

export async function updateLayer(
  id: string,
  input: { content?: string; target_ref?: string },
): Promise<PromptLayer> {
  const res = await fetch(`/api/prompt-layers/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(input),
  })
  return handleResponse<PromptLayer>(res)
}

export async function fetchVersions(layerId: string): Promise<PromptVersion[]> {
  const res = await fetch(`/api/prompt-layers/${layerId}/versions`)
  return handleResponse<PromptVersion[]>(res)
}

export async function rollbackVersion(layerId: string, version: number): Promise<PromptLayer> {
  const res = await fetch(`/api/prompt-layers/${layerId}/rollback`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ version }),
  })
  return handleResponse<PromptLayer>(res)
}

export async function fetchDiff(
  layerId: string,
  v1: number,
  v2: number,
): Promise<DiffResult> {
  const res = await fetch(`/api/prompt-layers/${layerId}/versions/diff?v1=${v1}&v2=${v2}`)
  return handleResponse<DiffResult>(res)
}

export async function fetchMergedPrompt(
  projectId: string,
  variables?: Record<string, string>,
): Promise<MergedPromptResponse> {
  const params = variables ? `?variables=${encodeURIComponent(JSON.stringify(variables))}` : ''
  const res = await fetch(`/api/projects/${projectId}/merged-prompt${params}`)
  return handleResponse<MergedPromptResponse>(res)
}
