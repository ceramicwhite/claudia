import { ReactElement } from 'react'
import { render, RenderOptions } from '@testing-library/react'
import { vi } from 'vitest'

// Custom render function that can include providers
export function renderWithProviders(
  ui: ReactElement,
  options?: Omit<RenderOptions, 'queries'>
) {
  return render(ui, {
    ...options,
  })
}

// Mock Tauri invoke responses
export function mockInvoke(command: string, response: any) {
  const { invoke } = vi.mocked(await import('@tauri-apps/api'))
  invoke.mockImplementation((cmd: string) => {
    if (cmd === command) {
      return Promise.resolve(response)
    }
    return Promise.reject(new Error(`Unknown command: ${cmd}`))
  })
}

// Mock Tauri event listeners
export function mockEventListener(event: string, callback: (payload: any) => void) {
  const { listen } = vi.mocked(await import('@tauri-apps/api/event'))
  listen.mockImplementation((evt: string, cb: any) => {
    if (evt === event) {
      // Return a mock unlisten function
      return Promise.resolve(() => {})
    }
    return Promise.resolve(() => {})
  })
}

// Helper to wait for async updates
export const waitForAsync = () => new Promise(resolve => setTimeout(resolve, 0))

// Re-export everything from React Testing Library
export * from '@testing-library/react'
export { default as userEvent } from '@testing-library/user-event'