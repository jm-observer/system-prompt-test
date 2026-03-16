const API = '/api'

async function handleResponse<T>(res: Response): Promise<T> {
  if (!res.ok) {
    const body = await res.text().catch(() => '')
    throw new Error(body || `HTTP ${res.status}`)
  }
  return res.json()
}

export interface RunReport {
  id: string
  run_id: string
  total_latency_ms: number
  total_tokens: number
  estimated_cost_usd: number
  assertion_passed_count: number
  assertion_failed_count: number
  failure_reason: string | null
  created_at: string
}

export interface ReportSummary {
  run_id: string
  status: string
  model_name: string
  latency_ms: number
  total_tokens: number
  cost_usd: number
  assertions_passed: number
  assertions_failed: number
}

export interface ModelPricing {
  id: string
  model_id: string
  input_1k_tokens_usd: number
  output_1k_tokens_usd: number
  updated_at: string
}

export interface AuditLog {
  id: string
  user_id: string | null
  action: string
  resource_type: string
  resource_id: string | null
  metadata: string | null
  created_at: string
}

export const fetchRunReport = (runId: string) =>
  fetch(`${API}/runs/${runId}/report`).then(r => handleResponse<RunReport>(r))

export const fetchReportsSummary = (promptVersionId?: string) => {
  const url = promptVersionId 
    ? `${API}/reports/summary?prompt_version_id=${promptVersionId}`
    : `${API}/reports/summary`
  return fetch(url).then(r => handleResponse<ReportSummary[]>(r))
}

export const fetchAuditLogs = () =>
  fetch(`${API}/audit-logs`).then(r => handleResponse<AuditLog[]>(r))

export const fetchModelPricing = (modelId: string) =>
  fetch(`${API}/models/${modelId}/pricing`).then(r => handleResponse<ModelPricing>(r))

export const updateModelPricing = async (modelId: string, data: { input_1k_tokens_usd: number, output_1k_tokens_usd: number }) => {
  const res = await fetch(`${API}/models/${modelId}/pricing`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  })
  if (!res.ok) {
    const body = await res.text().catch(() => '')
    throw new Error(body || `HTTP ${res.status}`)
  }
}
