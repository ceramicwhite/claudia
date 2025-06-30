import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { invoke } from '@tauri-apps/api'
import { listen } from '@tauri-apps/api/event'
import React from 'react'

// Example component that uses Tauri APIs
function SessionManager() {
  const [sessions, setSessions] = React.useState<any[]>([])
  const [loading, setLoading] = React.useState(false)
  const [error, setError] = React.useState<string | null>(null)

  const loadSessions = async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await invoke('get_sessions')
      setSessions(result as any[])
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load sessions')
    } finally {
      setLoading(false)
    }
  }

  React.useEffect(() => {
    loadSessions()

    // Listen for session updates
    const unlisten = listen('session-update', (event) => {
      setSessions(prev => [...prev, event.payload])
    })

    return () => {
      unlisten.then(fn => fn())
    }
  }, [])

  return (
    <div>
      {loading && <div>Loading sessions...</div>}
      {error && <div role="alert">Error: {error}</div>}
      <button onClick={loadSessions}>Refresh Sessions</button>
      <ul>
        {sessions.map((session, index) => (
          <li key={index}>{session.name}</li>
        ))}
      </ul>
    </div>
  )
}

describe('Tauri Integration Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('loads sessions on mount', async () => {
    const mockSessions = [
      { name: 'Session 1', id: '1' },
      { name: 'Session 2', id: '2' },
    ]
    
    vi.mocked(invoke).mockResolvedValueOnce(mockSessions)
    vi.mocked(listen).mockResolvedValueOnce(() => {})

    render(<SessionManager />)

    // Initially shows loading
    expect(screen.getByText('Loading sessions...')).toBeInTheDocument()

    // Wait for sessions to load
    await waitFor(() => {
      expect(screen.queryByText('Loading sessions...')).not.toBeInTheDocument()
    })

    // Check that invoke was called correctly
    expect(invoke).toHaveBeenCalledWith('get_sessions')

    // Sessions should be displayed
    expect(screen.getByText('Session 1')).toBeInTheDocument()
    expect(screen.getByText('Session 2')).toBeInTheDocument()
  })

  it('handles errors gracefully', async () => {
    const error = new Error('Database connection failed')
    vi.mocked(invoke).mockRejectedValueOnce(error)
    vi.mocked(listen).mockResolvedValueOnce(() => {})

    render(<SessionManager />)

    await waitFor(() => {
      expect(screen.getByRole('alert')).toHaveTextContent('Error: Database connection failed')
    })
  })

  it('refreshes sessions on button click', async () => {
    const user = userEvent.setup()
    const initialSessions = [{ name: 'Session 1', id: '1' }]
    const updatedSessions = [
      { name: 'Session 1', id: '1' },
      { name: 'Session 2', id: '2' },
    ]

    vi.mocked(invoke)
      .mockResolvedValueOnce(initialSessions)
      .mockResolvedValueOnce(updatedSessions)
    vi.mocked(listen).mockResolvedValueOnce(() => {})

    render(<SessionManager />)

    await waitFor(() => {
      expect(screen.getByText('Session 1')).toBeInTheDocument()
    })

    // Click refresh button
    const refreshButton = screen.getByRole('button', { name: /refresh sessions/i })
    await user.click(refreshButton)

    await waitFor(() => {
      expect(screen.getByText('Session 2')).toBeInTheDocument()
    })

    expect(invoke).toHaveBeenCalledTimes(2)
  })

  it('listens for session updates', async () => {
    const initialSessions = [{ name: 'Session 1', id: '1' }]
    let eventCallback: ((event: any) => void) | null = null

    vi.mocked(invoke).mockResolvedValueOnce(initialSessions)
    vi.mocked(listen).mockImplementation(async (event, callback) => {
      if (event === 'session-update') {
        eventCallback = callback
      }
      return () => {}
    })

    render(<SessionManager />)

    await waitFor(() => {
      expect(screen.getByText('Session 1')).toBeInTheDocument()
    })

    // Simulate a session update event
    await waitFor(() => {
      if (eventCallback) {
        eventCallback({ payload: { name: 'New Session', id: '3' } })
      }
    })

    await waitFor(() => {
      expect(screen.getByText('New Session')).toBeInTheDocument()
    })
  })
})