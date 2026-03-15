export interface Project {
  id: string
  name: string
  description: string
  created_at: string
  updated_at: string
}

export interface CreateProjectInput {
  name: string
  description?: string
}

export interface UpdateProjectInput {
  name?: string
  description?: string
}

const BASE = '/api/projects'

async function handleResponse<T>(res: Response): Promise<T> {
  if (!res.ok) {
    const text = await res.text()
    throw new Error(`API error ${res.status}: ${text}`)
  }
  if (res.status === 204) return undefined as T
  return res.json()
}

export async function fetchProjects(): Promise<Project[]> {
  const res = await fetch(BASE)
  return handleResponse<Project[]>(res)
}

export async function fetchProject(id: string): Promise<Project> {
  const res = await fetch(`${BASE}/${id}`)
  return handleResponse<Project>(res)
}

export async function createProject(input: CreateProjectInput): Promise<Project> {
  const res = await fetch(BASE, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(input),
  })
  return handleResponse<Project>(res)
}

export async function updateProject(id: string, input: UpdateProjectInput): Promise<Project> {
  const res = await fetch(`${BASE}/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(input),
  })
  return handleResponse<Project>(res)
}

export async function deleteProject(id: string): Promise<void> {
  const res = await fetch(`${BASE}/${id}`, { method: 'DELETE' })
  return handleResponse<void>(res)
}
