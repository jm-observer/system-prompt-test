const API = '/api'

async function handleResponse<T>(res: Response): Promise<T> {
  if (!res.ok) {
    const body = await res.text().catch(() => '')
    throw new Error(body || `HTTP ${res.status}`)
  }
  return res.json()
}

// Provider types
export interface ProviderResponse {
  id: string
  name: string
  api_type: string
  base_url: string
  api_key_masked: string
  created_at: string
  updated_at: string
}

export interface CreateProviderRequest {
  name: string
  api_type: string
  base_url: string
  api_key: string
}

export interface UpdateProviderRequest {
  name?: string
  base_url?: string
  api_key?: string
}

// Model types
export interface AiModel {
  id: string
  provider_id: string
  model_name: string
  capabilities: string
  is_active: boolean
  created_at: string
}

export interface CreateModelRequest {
  model_name: string
  capabilities?: string
}

export interface UpdateModelRequest {
  model_name?: string
  capabilities?: string
  is_active?: boolean
}

// Provider API
export const fetchProviders = () =>
  fetch(`${API}/providers`).then(r => handleResponse<ProviderResponse[]>(r))

export const createProvider = (data: CreateProviderRequest) =>
  fetch(`${API}/providers`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  }).then(r => handleResponse<ProviderResponse>(r))

export const updateProvider = (id: string, data: UpdateProviderRequest) =>
  fetch(`${API}/providers/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  }).then(r => handleResponse<ProviderResponse>(r))

export const deleteProvider = async (id: string) => {
  const res = await fetch(`${API}/providers/${id}`, { method: 'DELETE' })
  if (!res.ok) throw new Error(`Failed to delete provider: HTTP ${res.status}`)
}

// Model API
export const fetchModels = (providerId: string) =>
  fetch(`${API}/providers/${providerId}/models`).then(r => handleResponse<AiModel[]>(r))

export const fetchAllModels = () =>
  fetch(`${API}/models`).then(r => handleResponse<AiModel[]>(r))

export const createModel = (providerId: string, data: CreateModelRequest) =>
  fetch(`${API}/providers/${providerId}/models`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  }).then(r => handleResponse<AiModel>(r))

export const updateModel = (id: string, data: UpdateModelRequest) =>
  fetch(`${API}/models/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  }).then(r => handleResponse<AiModel>(r))

export const deleteModel = async (id: string) => {
  const res = await fetch(`${API}/models/${id}`, { method: 'DELETE' })
  if (!res.ok) throw new Error(`Failed to delete model: HTTP ${res.status}`)
}
