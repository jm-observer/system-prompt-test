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

  eventSource.onmessage = (e) => {
    try {
      const event: StreamEvent = JSON.parse(e.data)
      onEvent(event)
      if (event.event_type === 'done' || event.event_type === 'error') {
        eventSource.close()
      }
    } catch {
      // ignore parse errors
    }
  }

  eventSource.onerror = () => {
    onError?.(new Error('SSE connection error'))
    eventSource.close()
  }

  return () => eventSource.close()
}
