export interface Prompt {
  id: string
  name: string
  content: string
  created_at: string
  updated_at: string
}

export interface CreatePromptInput {
  name: string
  content?: string
}

export interface UpdatePromptInput {
  name?: string
  content?: string
}

const BASE = '/api/prompts'

async function handleResponse<T>(res: Response): Promise<T> {
  if (!res.ok) {
    const text = await res.text()
    throw new Error(`API error ${res.status}: ${text}`)
  }
  if (res.status === 204) return undefined as T
  return res.json()
}

export async function fetchPrompts(): Promise<Prompt[]> {
  const res = await fetch(BASE)
  return handleResponse<Prompt[]>(res)
}

export async function fetchPrompt(id: string): Promise<Prompt> {
  const res = await fetch(`${BASE}/${id}`)
  return handleResponse<Prompt>(res)
}

export async function createPrompt(input: CreatePromptInput): Promise<Prompt> {
  const res = await fetch(BASE, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(input),
  })
  return handleResponse<Prompt>(res)
}

export async function updatePrompt(id: string, input: UpdatePromptInput): Promise<Prompt> {
  const res = await fetch(`${BASE}/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(input),
  })
  return handleResponse<Prompt>(res)
}

export async function deletePrompt(id: string): Promise<void> {
  const res = await fetch(`${BASE}/${id}`, { method: 'DELETE' })
  return handleResponse<void>(res)
}
