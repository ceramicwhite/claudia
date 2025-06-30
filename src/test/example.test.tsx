import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { invoke } from '@tauri-apps/api'

// Simple component for testing
function TestButton({ onClick }: { onClick: () => void }) {
  return (
    <button onClick={onClick} className="bg-blue-500 text-white p-2 rounded">
      Click me
    </button>
  )
}

describe('Example Test Suite', () => {
  it('should render a button with correct text', () => {
    const mockClick = vi.fn()
    render(<TestButton onClick={mockClick} />)
    
    const button = screen.getByRole('button', { name: /click me/i })
    expect(button).toBeInTheDocument()
    expect(button).toHaveClass('bg-blue-500', 'text-white')
  })

  it('should handle click events', async () => {
    const user = userEvent.setup()
    const mockClick = vi.fn()
    render(<TestButton onClick={mockClick} />)
    
    const button = screen.getByRole('button', { name: /click me/i })
    await user.click(button)
    
    expect(mockClick).toHaveBeenCalledTimes(1)
  })

  it('should mock Tauri API calls', async () => {
    const mockResponse = { data: 'test' }
    vi.mocked(invoke).mockResolvedValueOnce(mockResponse)
    
    const result = await invoke('test_command', { arg: 'value' })
    
    expect(invoke).toHaveBeenCalledWith('test_command', { arg: 'value' })
    expect(result).toEqual(mockResponse)
  })
})