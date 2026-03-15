const API = '/api'

async function handleResponse<T>(res: Response): Promise<T> {
  if (!res.ok) throw new Error(`HTTP ${res.status}`)
  return res.json()
}

export interface TestCase {
  id: string
  project_id: string
  name: string
  user_message: string
  config: string
  created_at: string
  updated_at: string
}

export interface CreateTestCaseRequest {
  name: string
  user_message: string
  config?: string
}

export interface UpdateTestCaseRequest {
  name?: string
  user_message?: string
  config?: string
}

export interface Run {
  id: string
  test_case_id: string
  model_id: string
  status: string
  system_prompt: string
  started_at: string | null
  finished_at: string | null
  created_at: string
}

export interface RunResult {
  id: string
  run_id: string
  response_text: string
  token_usage: string
  latency_ms: number | null
  raw_response: string
  error_message: string | null
  created_at: string
}

export interface RunWithResult extends Run {
  result: RunResult | null
}

export interface Assertion {
  id: string
  test_case_id: string
  assertion_type: string
  config: string
  created_at: string
}

export interface AssertionResult {
  id: string
  run_id: string
  assertion_id: string
  passed: boolean
  evidence: string | null
  created_at: string
}

export interface RunRequest {
  model_ids: string[]
  variables?: Record<string, string>
}

// Test Case API
export const fetchTestCases = (projectId: string) =>
  fetch(`${API}/projects/${projectId}/test-cases`).then(r => handleResponse<TestCase[]>(r))

export const createTestCase = (projectId: string, data: CreateTestCaseRequest) =>
  fetch(`${API}/projects/${projectId}/test-cases`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  }).then(r => handleResponse<TestCase>(r))

export const updateTestCase = (id: string, data: UpdateTestCaseRequest) =>
  fetch(`${API}/test-cases/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  }).then(r => handleResponse<TestCase>(r))

export const deleteTestCase = (id: string) =>
  fetch(`${API}/test-cases/${id}`, { method: 'DELETE' })

// Run API
export const createRun = (testCaseId: string, data: RunRequest) =>
  fetch(`${API}/test-cases/${testCaseId}/run`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  }).then(r => handleResponse<Run[]>(r))

export const fetchRuns = (testCaseId: string) =>
  fetch(`${API}/test-cases/${testCaseId}/runs`).then(r => handleResponse<RunWithResult[]>(r))

export const fetchRun = (runId: string) =>
  fetch(`${API}/runs/${runId}`).then(r => handleResponse<RunWithResult>(r))

// Assertion API
export const fetchAssertions = (testCaseId: string) =>
  fetch(`${API}/test-cases/${testCaseId}/assertions`).then(r => handleResponse<Assertion[]>(r))

export const createAssertion = (testCaseId: string, data: { assertion_type: string, config: string }) =>
  fetch(`${API}/test-cases/${testCaseId}/assertions`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  }).then(r => handleResponse<Assertion>(r))

export const deleteAssertion = (id: string) =>
  fetch(`${API}/assertions/${id}`, { method: 'DELETE' })

export const fetchAssertionResults = (runId: string) =>
  fetch(`${API}/runs/${runId}/assertion-results`).then(r => handleResponse<AssertionResult[]>(r))
