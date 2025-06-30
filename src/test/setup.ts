import '@testing-library/jest-dom'
import { vi } from 'vitest'

// Mock Tauri API
vi.mock('@tauri-apps/api', () => ({
  invoke: vi.fn(),
  convertFileSrc: vi.fn((path: string) => path),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
  emit: vi.fn(),
  once: vi.fn(),
}))

vi.mock('@tauri-apps/api/window', () => ({
  currentMonitor: vi.fn(),
  primaryMonitor: vi.fn(),
  availableMonitors: vi.fn(),
  appWindow: {
    maximize: vi.fn(),
    minimize: vi.fn(),
    close: vi.fn(),
    show: vi.fn(),
    hide: vi.fn(),
    setFullscreen: vi.fn(),
    isFullscreen: vi.fn(),
    setTitle: vi.fn(),
    getTitle: vi.fn(),
    listen: vi.fn(),
    emit: vi.fn(),
  },
}))

vi.mock('@tauri-apps/api/path', () => ({
  appDataDir: vi.fn(() => Promise.resolve('/mock/app-data')),
  appConfigDir: vi.fn(() => Promise.resolve('/mock/app-config')),
  appCacheDir: vi.fn(() => Promise.resolve('/mock/app-cache')),
  appLogDir: vi.fn(() => Promise.resolve('/mock/app-log')),
  homeDir: vi.fn(() => Promise.resolve('/mock/home')),
  dataDir: vi.fn(() => Promise.resolve('/mock/data')),
  desktopDir: vi.fn(() => Promise.resolve('/mock/desktop')),
  documentDir: vi.fn(() => Promise.resolve('/mock/documents')),
  downloadDir: vi.fn(() => Promise.resolve('/mock/downloads')),
  join: vi.fn((...parts: string[]) => Promise.resolve(parts.join('/'))),
  basename: vi.fn((path: string) => Promise.resolve(path.split('/').pop() || '')),
  dirname: vi.fn((path: string) => Promise.resolve(path.split('/').slice(0, -1).join('/'))),
}))

// Mock Tauri plugins
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
  save: vi.fn(),
  message: vi.fn(),
  ask: vi.fn(),
  confirm: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-shell', () => ({
  open: vi.fn(),
  Command: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-global-shortcut', () => ({
  register: vi.fn(),
  unregister: vi.fn(),
  isRegistered: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-opener', () => ({
  open: vi.fn(),
}))

// Mock window.matchMedia for Tailwind CSS dark mode tests
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(), // deprecated
    removeListener: vi.fn(), // deprecated
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
})

// Mock IntersectionObserver for components that use it
global.IntersectionObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}))

// Mock ResizeObserver for components that use it
global.ResizeObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}))

// Suppress console errors during tests (optional)
const originalError = console.error
beforeAll(() => {
  console.error = (...args: any[]) => {
    if (
      typeof args[0] === 'string' &&
      args[0].includes('Consider adding an error boundary')
    ) {
      return
    }
    originalError(...args)
  }
})

afterAll(() => {
  console.error = originalError
})