export interface StreamEvent {
  event_type: string
  content: string | null
  token_usage: { prompt_tokens: number; completion_tokens: number; total_tokens: number } | null
  error: string | null
}

export function streamRun(
  runId: string,
  onEvent: (event: StreamEvent) => void,
  onError?: (error: Error) => void,
): () => void {
  const eventSource = new EventSource(`/api/runs/${runId}/stream`)
  let closed = false

  const cleanup = () => {
    if (!closed) {
      closed = true
      eventSource.close()
    }
  }

  eventSource.onmessage = (e) => {
    if (closed) return
    try {
      const event: StreamEvent = JSON.parse(e.data)
      onEvent(event)
      if (event.event_type === 'done' || event.event_type === 'error') {
        cleanup()
      }
    } catch (err) {
      console.warn('Failed to parse SSE event:', err)
    }
  }

  eventSource.onerror = () => {
    if (closed) return
    onError?.(new Error('SSE connection error'))
    cleanup()
  }

  return cleanup
}
