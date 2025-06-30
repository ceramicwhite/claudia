import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { DateTimePicker } from './date-time-picker'

// Mock ReactDOM.createPortal
vi.mock('react-dom', async () => {
  const actual = await vi.importActual('react-dom')
  return {
    ...actual,
    createPortal: (children: any) => children,
  }
})

describe('DateTimePicker', () => {
  const mockOnChange = vi.fn()
  
  beforeEach(() => {
    vi.clearAllMocks()
    // Fix for framer-motion matchMedia issue in tests
    global.window.matchMedia = vi.fn().mockImplementation(() => ({
      matches: false,
      media: '',
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    }))
  })

  describe('Basic functionality', () => {
    it('should render with placeholder', () => {
      render(<DateTimePicker placeholder="Select a date" onChange={mockOnChange} />)
      
      const button = screen.getByRole('button')
      expect(button).toHaveTextContent('Select a date')
    })

    it('should render with value', () => {
      render(<DateTimePicker value="2024-06-20T15:45:00Z" onChange={mockOnChange} />)
      
      const button = screen.getByRole('button')
      expect(button).toHaveTextContent('Jun 20, 2024')
    })

    it('should show calendar icon', () => {
      render(<DateTimePicker onChange={mockOnChange} />)
      
      const button = screen.getByRole('button')
      const icon = button.querySelector('svg')
      expect(icon).toBeInTheDocument()
    })

    it('should handle disabled state', () => {
      render(<DateTimePicker disabled onChange={mockOnChange} />)
      
      const button = screen.getByRole('button')
      expect(button).toBeDisabled()
    })
  })

  describe('Popover interaction', () => {
    it('should open popover when button is clicked', async () => {
      const user = userEvent.setup()
      render(<DateTimePicker onChange={mockOnChange} />)
      
      const button = screen.getByRole('button')
      await user.click(button)
      
      await waitFor(() => {
        expect(screen.getByText('Select Date & Time')).toBeInTheDocument()
        expect(screen.getByText('Date')).toBeInTheDocument()
        expect(screen.getByText('Time')).toBeInTheDocument()
      })
    })

    it('should show clear button when value exists', async () => {
      const user = userEvent.setup()
      render(<DateTimePicker value="2024-01-15T10:30:00Z" onChange={mockOnChange} />)
      
      const button = screen.getByRole('button')
      await user.click(button)
      
      await waitFor(() => {
        const clearButton = screen.getByRole('button', { name: 'Clear Schedule' })
        expect(clearButton).not.toBeDisabled()
      })
    })

    it('should disable clear button when no value', async () => {
      const user = userEvent.setup()
      render(<DateTimePicker onChange={mockOnChange} />)
      
      const button = screen.getByRole('button')
      await user.click(button)
      
      await waitFor(() => {
        const clearButton = screen.getByRole('button', { name: 'Clear Schedule' })
        expect(clearButton).toBeDisabled()
      })
    })

    it('should clear value when Clear Schedule is clicked', async () => {
      const user = userEvent.setup()
      render(<DateTimePicker value="2024-01-15T10:30:00Z" onChange={mockOnChange} />)
      
      const button = screen.getByRole('button')
      await user.click(button)
      
      await waitFor(() => {
        expect(screen.getByText('Select Date & Time')).toBeInTheDocument()
      })
      
      const clearButton = screen.getByRole('button', { name: 'Clear Schedule' })
      await user.click(clearButton)
      
      expect(mockOnChange).toHaveBeenCalledWith(undefined)
    })
  })

  describe('Value formatting', () => {
    it('should format date correctly', () => {
      const testCases = [
        { value: '2024-01-15T10:30:00Z', expected: 'Jan 15, 2024' },
        { value: '2024-12-25T08:00:00Z', expected: 'Dec 25, 2024' },
        { value: '2024-07-04T23:59:00Z', expected: 'Jul' }, // Contains Jul
      ]
      
      testCases.forEach(({ value, expected }) => {
        const { unmount } = render(<DateTimePicker value={value} onChange={mockOnChange} />)
        const button = screen.getByRole('button')
        expect(button.textContent).toContain(expected)
        unmount()
      })
    })

    it('should handle empty value', () => {
      render(<DateTimePicker onChange={mockOnChange} />)
      
      const button = screen.getByRole('button')
      expect(button).toHaveTextContent('Select date and time')
    })
  })

  describe('onChange callback', () => {
    it('should call onChange with ISO string when date is selected', async () => {
      const user = userEvent.setup()
      render(<DateTimePicker onChange={mockOnChange} />)
      
      const button = screen.getByRole('button')
      await user.click(button)
      
      // Wait for popover to open and onChange to be called
      await waitFor(() => {
        expect(screen.getByText('Select Date & Time')).toBeInTheDocument()
      })
      
      // onChange should be called when popover opens (initializes with current time)
      await waitFor(() => {
        expect(mockOnChange).toHaveBeenCalled()
        const callArg = mockOnChange.mock.calls[0][0]
        expect(callArg).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}/)
      })
    })
  })

  describe('Dropdown interactions', () => {
    it('should have month dropdown', async () => {
      const user = userEvent.setup()
      render(<DateTimePicker onChange={mockOnChange} />)
      
      const button = screen.getByRole('button')
      await user.click(button)
      
      await waitFor(() => {
        expect(screen.getByText('Select Date & Time')).toBeInTheDocument()
      })
      
      // Find month dropdown button
      const monthButtons = screen.getAllByRole('button')
      const monthButton = monthButtons.find(btn => 
        btn.textContent && /January|February|March|April|May|June|July|August|September|October|November|December/.test(btn.textContent)
      )
      
      expect(monthButton).toBeInTheDocument()
    })

    it('should show current date values in dropdowns', async () => {
      const user = userEvent.setup()
      const currentDate = new Date()
      render(<DateTimePicker onChange={mockOnChange} />)
      
      const button = screen.getByRole('button')
      await user.click(button)
      
      await waitFor(() => {
        expect(screen.getByText('Select Date & Time')).toBeInTheDocument()
      })
      
      // Check that current month is shown
      const monthName = currentDate.toLocaleString('default', { month: 'long' })
      const monthButtons = screen.getAllByRole('button')
      const hasCurrentMonth = monthButtons.some(btn => btn.textContent?.includes(monthName))
      expect(hasCurrentMonth).toBe(true)
    })
  })

  describe('Timezone display', () => {
    it('should show timezone information', async () => {
      const user = userEvent.setup()
      render(<DateTimePicker value="2024-01-15T10:30:00Z" onChange={mockOnChange} />)
      
      const button = screen.getByRole('button')
      await user.click(button)
      
      await waitFor(() => {
        expect(screen.getByText(/Timezone:/)).toBeInTheDocument()
      })
    })

    it('should show scheduled time', async () => {
      const user = userEvent.setup()
      render(<DateTimePicker value="2024-01-15T10:30:00Z" onChange={mockOnChange} />)
      
      const button = screen.getByRole('button')
      await user.click(button)
      
      await waitFor(() => {
        expect(screen.getByText(/Scheduled for:/)).toBeInTheDocument()
      })
    })
  })

  describe('Accessibility', () => {
    it('should have accessible button', () => {
      render(<DateTimePicker onChange={mockOnChange} />)
      
      const button = screen.getByRole('button')
      expect(button).toBeInTheDocument()
    })

    it('should have proper labels in popover', async () => {
      const user = userEvent.setup()
      render(<DateTimePicker onChange={mockOnChange} />)
      
      const button = screen.getByRole('button')
      await user.click(button)
      
      await waitFor(() => {
        expect(screen.getByText('Select Date & Time')).toBeInTheDocument()
        expect(screen.getByText('Date')).toBeInTheDocument()
        expect(screen.getByText('Time')).toBeInTheDocument()
      })
    })
  })
})