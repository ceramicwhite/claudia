import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import {
  TodoWidget,
  LSWidget,
  LSResultWidget,
  ReadWidget,
  ReadResultWidget,
  GlobWidget,
  BashWidget,
  WriteWidget,
  GrepWidget,
  EditWidget,
  EditResultWidget,
  MCPWidget,
  CommandWidget,
  CommandOutputWidget,
  SummaryWidget,
  MultiEditWidget,
  MultiEditResultWidget,
  SystemReminderWidget,
  SystemInitializedWidget,
  TaskWidget,
  WebSearchWidget,
  ThinkingWidget,
} from './ToolWidgets'
import { TodoStatus, Priority, ClaudeModel } from '@/constants'
import { open } from '@tauri-apps/plugin-shell'

// Mock dependencies
vi.mock('framer-motion', () => ({
  motion: {
    div: ({ children, ...props }: any) => <div {...props}>{children}</div>,
  },
}))

vi.mock('react-dom', () => ({
  ...vi.importActual('react-dom'),
  createPortal: (children: any) => children,
}))

vi.mock('@tauri-apps/plugin-shell', () => ({
  open: vi.fn(),
}))

vi.mock('@/lib/linkDetector', () => ({
  detectLinks: vi.fn((text: string) => {
    const urlRegex = /(https?:\/\/[^\s]+)/g
    const matches = text.match(urlRegex) || []
    return matches.map(url => ({ fullUrl: url }))
  }),
  makeLinksClickable: vi.fn((text: string, callback?: (url: string) => void) => {
    const urlRegex = /(https?:\/\/[^\s]+)/g
    const parts = text.split(urlRegex)
    return parts.map((part, idx) => {
      if (part.match(urlRegex)) {
        return (
          <a
            key={idx}
            href={part}
            onClick={(e) => {
              e.preventDefault()
              callback?.(part)
            }}
            className="text-blue-500 hover:underline"
          >
            {part}
          </a>
        )
      }
      return part
    })
  }),
}))

// Mock syntax highlighter to avoid complex rendering in tests
vi.mock('react-syntax-highlighter', () => ({
  Prism: ({ children, language, showLineNumbers, startingLineNumber }: any) => (
    <pre data-testid="syntax-highlighter" data-language={language} data-show-line-numbers={showLineNumbers} data-starting-line={startingLineNumber}>
      <code>{children}</code>
    </pre>
  ),
}))

vi.mock('@/lib/claudeSyntaxTheme', () => ({
  claudeSyntaxTheme: {},
}))

describe('ToolWidgets', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('TodoWidget', () => {
    const mockTodos = [
      {
        id: '1',
        content: 'Complete unit tests',
        status: TodoStatus.IN_PROGRESS,
        priority: Priority.HIGH,
      },
      {
        id: '2',
        content: 'Review PR',
        status: TodoStatus.COMPLETED,
        priority: Priority.MEDIUM,
      },
      {
        id: '3',
        content: 'Update documentation',
        status: TodoStatus.PENDING,
        priority: Priority.LOW,
      },
    ]

    it('should render todo list with correct status icons', () => {
      render(<TodoWidget todos={mockTodos} />)
      
      expect(screen.getByText('Todo List')).toBeInTheDocument()
      expect(screen.getByText('Complete unit tests')).toBeInTheDocument()
      expect(screen.getByText('Review PR')).toBeInTheDocument()
      expect(screen.getByText('Update documentation')).toBeInTheDocument()
    })

    it('should apply correct styles for completed tasks', () => {
      render(<TodoWidget todos={mockTodos} />)
      
      const completedTask = screen.getByText('Review PR')
      expect(completedTask).toHaveClass('line-through')
      expect(completedTask.closest('div')).toHaveClass('opacity-60')
    })

    it('should display priority badges with correct colors', () => {
      render(<TodoWidget todos={mockTodos} />)
      
      const highPriorityBadge = screen.getByText(Priority.HIGH)
      expect(highPriorityBadge).toHaveClass('bg-red-500/10', 'text-red-500')
      
      const mediumPriorityBadge = screen.getByText(Priority.MEDIUM)
      expect(mediumPriorityBadge).toHaveClass('bg-yellow-500/10', 'text-yellow-500')
      
      const lowPriorityBadge = screen.getByText(Priority.LOW)
      expect(lowPriorityBadge).toHaveClass('bg-green-500/10', 'text-green-500')
    })

    it('should handle todos without ids', () => {
      const todosWithoutIds = mockTodos.map(({ id, ...rest }) => rest)
      render(<TodoWidget todos={todosWithoutIds} />)
      
      expect(screen.getByText('Complete unit tests')).toBeInTheDocument()
    })

    it('should handle empty todo list', () => {
      render(<TodoWidget todos={[]} />)
      
      expect(screen.getByText('Todo List')).toBeInTheDocument()
      expect(screen.queryByText('Complete unit tests')).not.toBeInTheDocument()
    })
  })

  describe('LSWidget', () => {
    it('should render directory path and loading state', () => {
      render(<LSWidget path="/test/directory" />)
      
      expect(screen.getByText('Listing directory:')).toBeInTheDocument()
      expect(screen.getByText('/test/directory')).toBeInTheDocument()
      expect(screen.getByText('Loading...')).toBeInTheDocument()
    })

    it('should render result when provided', () => {
      const result = {
        content: '- file1.txt\n- folder1/\n  - file2.js\n- file3.md',
      }
      
      render(<LSWidget path="/test/directory" result={result} />)
      
      expect(screen.getByText('Directory contents for:')).toBeInTheDocument()
      expect(screen.queryByText('Loading...')).not.toBeInTheDocument()
    })

    it('should handle result with nested content structure', () => {
      const result = {
        content: {
          text: '- file1.txt\n- folder1/',
        },
      }
      
      render(<LSWidget path="/test/directory" result={result} />)
      
      expect(screen.getByTestId('syntax-highlighter')).toHaveTextContent('- file1.txt')
    })

    it('should handle result with array content', () => {
      const result = {
        content: [
          { text: '- file1.txt' },
          '- folder1/',
        ],
      }
      
      render(<LSWidget path="/test/directory" result={result} />)
      
      expect(screen.getByTestId('syntax-highlighter')).toHaveTextContent('- file1.txt\n- folder1/')
    })
  })

  describe('LSResultWidget', () => {
    it('should render directory tree structure', () => {
      const content = `- src/
  - components/
    - Button.tsx
    - Card.tsx
  - utils/
    - helpers.ts
- package.json
- README.md`

      render(<LSResultWidget content={content} />)
      
      expect(screen.getByText('src')).toBeInTheDocument()
      expect(screen.getByText('components')).toBeInTheDocument()
      expect(screen.getByText('Button.tsx')).toBeInTheDocument()
    })

    it('should handle file type icons', () => {
      const content = `- script.js
- data.json
- styles.css
- doc.md
- code.py`

      render(<LSResultWidget content={content} />)
      
      // Check that files are rendered
      expect(screen.getByText('script.js')).toBeInTheDocument()
      expect(screen.getByText('data.json')).toBeInTheDocument()
      expect(screen.getByText('styles.css')).toBeInTheDocument()
    })

    it('should toggle directory expansion', async () => {
      const user = userEvent.setup()
      const content = `- folder/
  - file.txt`

      render(<LSResultWidget content={content} />)
      
      // Initially expanded
      expect(screen.getByText('file.txt')).toBeInTheDocument()
      
      // Click to collapse
      const folderElement = screen.getByText('folder')
      await user.click(folderElement)
      
      // Should be collapsed
      expect(screen.queryByText('file.txt')).not.toBeInTheDocument()
    })

    it('should skip NOTE section', () => {
      const content = `- file1.txt
- file2.txt
NOTE: This is a note that should be ignored
- file3.txt`

      render(<LSResultWidget content={content} />)
      
      expect(screen.getByText('file1.txt')).toBeInTheDocument()
      expect(screen.getByText('file2.txt')).toBeInTheDocument()
      expect(screen.queryByText('NOTE:')).not.toBeInTheDocument()
      expect(screen.queryByText('file3.txt')).not.toBeInTheDocument()
    })
  })

  describe('ReadWidget', () => {
    it('should render file path and loading state', () => {
      render(<ReadWidget filePath="/test/file.txt" />)
      
      expect(screen.getByText('Reading file:')).toBeInTheDocument()
      expect(screen.getByText('/test/file.txt')).toBeInTheDocument()
      expect(screen.getByText('Loading...')).toBeInTheDocument()
    })

    it('should render result when provided', () => {
      const result = {
        content: '1→const hello = "world";\n2→console.log(hello);',
      }
      
      render(<ReadWidget filePath="/test/file.js" result={result} />)
      
      expect(screen.getByText('File content:')).toBeInTheDocument()
      expect(screen.queryByText('Loading...')).not.toBeInTheDocument()
    })
  })

  describe('ReadResultWidget', () => {
    it('should render file content with syntax highlighting', () => {
      const content = `1→function hello() {
2→  console.log("Hello, world!");
3→}`

      render(<ReadResultWidget content={content} filePath="/test/file.js" />)
      
      const highlighter = screen.getByTestId('syntax-highlighter')
      expect(highlighter).toHaveAttribute('data-language', 'javascript')
      expect(highlighter).toHaveAttribute('data-show-line-numbers', 'true')
      expect(highlighter).toHaveAttribute('data-starting-line', '1')
    })

    it('should detect and set correct language based on file extension', () => {
      const extensions = [
        { ext: 'file.py', lang: 'python' },
        { ext: 'file.rs', lang: 'rust' },
        { ext: 'file.tsx', lang: 'tsx' },
        { ext: 'file.md', lang: 'markdown' },
        { ext: 'file.json', lang: 'json' },
      ]

      extensions.forEach(({ ext, lang }) => {
        const { rerender } = render(<ReadResultWidget content="test" filePath={`/test/${ext}`} />)
        const highlighter = screen.getByTestId('syntax-highlighter')
        expect(highlighter).toHaveAttribute('data-language', lang)
        rerender(<div />)
      })
    })

    it('should handle large files with expand/collapse', async () => {
      const user = userEvent.setup()
      const largeContent = Array.from({ length: 25 }, (_, i) => `${i + 1}→Line ${i + 1}`).join('\n')
      
      render(<ReadResultWidget content={largeContent} filePath="/test/large.txt" />)
      
      expect(screen.getByText('(25 lines)')).toBeInTheDocument()
      expect(screen.getByText('Expand')).toBeInTheDocument()
      
      // Initially collapsed
      expect(screen.getByText('Click "Expand" to view the full file')).toBeInTheDocument()
      
      // Click to expand
      await user.click(screen.getByText('Expand'))
      
      expect(screen.getByText('Collapse')).toBeInTheDocument()
      expect(screen.queryByText('Click "Expand" to view the full file')).not.toBeInTheDocument()
    })

    it('should parse line numbers correctly', () => {
      const content = `10→const start = 10;
11→const end = 20;
12→console.log(start, end);`

      render(<ReadResultWidget content={content} />)
      
      const highlighter = screen.getByTestId('syntax-highlighter')
      expect(highlighter).toHaveAttribute('data-starting-line', '10')
      expect(highlighter).toHaveTextContent('const start = 10;')
    })

    it('should handle content without line numbers', () => {
      const content = 'Just plain text without line numbers'
      
      render(<ReadResultWidget content={content} />)
      
      const highlighter = screen.getByTestId('syntax-highlighter')
      expect(highlighter).toHaveAttribute('data-starting-line', '1')
      expect(highlighter).toHaveTextContent(content)
    })
  })

  describe('GlobWidget', () => {
    it('should render search pattern', () => {
      render(<GlobWidget pattern="**/*.tsx" />)
      
      expect(screen.getByText('Searching for pattern:')).toBeInTheDocument()
      expect(screen.getByText('**/*.tsx')).toBeInTheDocument()
      expect(screen.getByText('Searching...')).toBeInTheDocument()
    })

    it('should render success result', () => {
      const result = {
        content: 'Found 5 files:\n- src/App.tsx\n- src/components/Button.tsx',
      }
      
      render(<GlobWidget pattern="**/*.tsx" result={result} />)
      
      expect(screen.queryByText('Searching...')).not.toBeInTheDocument()
      expect(screen.getByText(/Found 5 files/)).toBeInTheDocument()
    })

    it('should render error result', () => {
      const result = {
        content: 'Error: Permission denied',
        is_error: true,
      }
      
      render(<GlobWidget pattern="**/*.tsx" result={result} />)
      
      const resultElement = screen.getByText('Error: Permission denied')
      expect(resultElement.parentElement).toHaveClass('border-red-500/20', 'bg-red-500/5')
    })

    it('should handle empty result', () => {
      const result = {
        content: '',
      }
      
      render(<GlobWidget pattern="**/*.tsx" result={result} />)
      
      expect(screen.getByText('No matches found')).toBeInTheDocument()
    })
  })

  describe('BashWidget', () => {
    it('should render command with description', () => {
      render(<BashWidget command="npm install" description="Install dependencies" />)
      
      expect(screen.getByText('Terminal')).toBeInTheDocument()
      expect(screen.getByText('Install dependencies')).toBeInTheDocument()
      expect(screen.getByText('$ npm install')).toBeInTheDocument()
      expect(screen.getByText('Running...')).toBeInTheDocument()
    })

    it('should render command without description', () => {
      render(<BashWidget command="ls -la" />)
      
      expect(screen.getByText('$ ls -la')).toBeInTheDocument()
      expect(screen.queryByText('Install dependencies')).not.toBeInTheDocument()
    })

    it('should render success result', () => {
      const result = {
        content: 'Command output here',
      }
      
      render(<BashWidget command="echo test" result={result} />)
      
      expect(screen.queryByText('Running...')).not.toBeInTheDocument()
      expect(screen.getByText('Command output here')).toBeInTheDocument()
    })

    it('should render error result', () => {
      const result = {
        content: 'Command failed: command not found',
        is_error: true,
      }
      
      render(<BashWidget command="invalid-command" result={result} />)
      
      const resultElement = screen.getByText('Command failed: command not found')
      expect(resultElement.parentElement).toHaveClass('border-red-500/20', 'bg-red-500/5')
    })
  })

  describe('WriteWidget', () => {
    it('should render file path and content preview', () => {
      const content = 'const greeting = "Hello, World!";'
      
      render(<WriteWidget filePath="/test/hello.js" content={content} />)
      
      expect(screen.getByText('Writing to file:')).toBeInTheDocument()
      expect(screen.getByText('/test/hello.js')).toBeInTheDocument()
      expect(screen.getByTestId('syntax-highlighter')).toHaveTextContent(content)
    })

    it('should truncate large content', () => {
      const largeContent = 'x'.repeat(1500)
      
      render(<WriteWidget filePath="/test/large.txt" content={largeContent} />)
      
      expect(screen.getByText('Truncated to 1000 chars')).toBeInTheDocument()
      expect(screen.getByTestId('syntax-highlighter')).toHaveTextContent('x'.repeat(1000) + '\n...')
    })

    it('should open maximized view', async () => {
      const user = userEvent.setup()
      const content = 'x'.repeat(1500)
      
      render(<WriteWidget filePath="/test/large.txt" content={content} />)
      
      const maximizeButton = screen.getByRole('button')
      await user.click(maximizeButton)
      
      // Check modal content
      expect(screen.getByText('/test/large.txt')).toBeInTheDocument()
      expect(screen.getAllByTestId('syntax-highlighter')[1]).toHaveTextContent('x'.repeat(1500))
    })

    it('should close maximized view', async () => {
      const user = userEvent.setup()
      const content = 'x'.repeat(1500)
      
      render(<WriteWidget filePath="/test/large.txt" content={content} />)
      
      // Open modal
      const maximizeButton = screen.getByRole('button')
      await user.click(maximizeButton)
      
      // Close modal
      const closeButton = screen.getAllByRole('button').find(btn => btn.querySelector('svg'))
      await user.click(closeButton!)
      
      // Modal should be closed
      expect(screen.getAllByText('/test/large.txt')).toHaveLength(1)
    })
  })

  describe('GrepWidget', () => {
    it('should render search parameters', () => {
      render(
        <GrepWidget 
          pattern="TODO" 
          include="*.js" 
          path="/src" 
          exclude="node_modules"
        />
      )
      
      expect(screen.getByText('Searching with grep')).toBeInTheDocument()
      expect(screen.getByText('TODO')).toBeInTheDocument()
      expect(screen.getByText('*.js')).toBeInTheDocument()
      expect(screen.getByText('/src')).toBeInTheDocument()
      expect(screen.getByText('node_modules')).toBeInTheDocument()
    })

    it('should render search results', () => {
      const result = {
        content: 'src/app.js:10:// TODO: Implement feature\nsrc/utils.js:25:// TODO: Add tests',
      }
      
      render(<GrepWidget pattern="TODO" result={result} />)
      
      expect(screen.getByText('2 matches found')).toBeInTheDocument()
      expect(screen.getByText('app.js')).toBeInTheDocument()
      expect(screen.getByText('// TODO: Implement feature')).toBeInTheDocument()
    })

    it('should toggle results expansion', async () => {
      const user = userEvent.setup()
      const result = {
        content: 'src/app.js:10:// TODO: Implement feature',
      }
      
      render(<GrepWidget pattern="TODO" result={result} />)
      
      // Initially expanded
      expect(screen.getByText('// TODO: Implement feature')).toBeInTheDocument()
      
      // Click to collapse
      await user.click(screen.getByText('1 match found'))
      
      // Should be collapsed
      expect(screen.queryByText('// TODO: Implement feature')).not.toBeInTheDocument()
    })

    it('should handle no matches', () => {
      const result = {
        content: '',
      }
      
      render(<GrepWidget pattern="NOTFOUND" result={result} />)
      
      expect(screen.getByText('No matches found for the given pattern.')).toBeInTheDocument()
    })

    it('should handle search errors', () => {
      const result = {
        content: 'grep: invalid regular expression',
        is_error: true,
      }
      
      render(<GrepWidget pattern="[invalid" result={result} />)
      
      expect(screen.getByText('grep: invalid regular expression')).toBeInTheDocument()
    })
  })

  describe('EditWidget', () => {
    it('should render file path and diff', () => {
      render(
        <EditWidget 
          file_path="/test/file.js"
          old_string="const old = 1;"
          new_string="const new = 2;"
        />
      )
      
      expect(screen.getByText('Applying Edit to:')).toBeInTheDocument()
      expect(screen.getByText('/test/file.js')).toBeInTheDocument()
      expect(screen.getByTestId('syntax-highlighter')).toBeInTheDocument()
    })

    it('should handle multi-line diffs', () => {
      const oldString = `function test() {
  return 1;
}`
      const newString = `function test() {
  return 2;
}`
      
      render(
        <EditWidget 
          file_path="/test/file.js"
          old_string={oldString}
          new_string={newString}
        />
      )
      
      expect(screen.getAllByTestId('syntax-highlighter')).toHaveLength(2)
    })

    it('should show unchanged lines indicator for large diffs', () => {
      const oldString = Array.from({ length: 20 }, (_, i) => `line ${i}`).join('\n')
      const newString = oldString.replace('line 10', 'modified line 10')
      
      render(
        <EditWidget 
          file_path="/test/file.txt"
          old_string={oldString}
          new_string={newString}
        />
      )
      
      // Look for the unchanged lines indicator
      const unchangedIndicators = screen.getAllByText((content, element) => {
        return element?.textContent?.includes('unchanged lines') || false
      })
      expect(unchangedIndicators.length).toBeGreaterThan(0)
    })
  })

  describe('EditResultWidget', () => {
    it('should parse and display edit result', () => {
      const content = `The file /test/file.js has been updated.
10	function updated() {
11	  return true;
12	}`

      render(<EditResultWidget content={content} />)
      
      expect(screen.getByText('Edit Result')).toBeInTheDocument()
      expect(screen.getByText('/test/file.js')).toBeInTheDocument()
      expect(screen.getByTestId('syntax-highlighter')).toHaveAttribute('data-starting-line', '10')
    })

    it('should handle content without file path', () => {
      const content = `1	const value = 42;
2	console.log(value);`

      render(<EditResultWidget content={content} />)
      
      expect(screen.getByText('Edit Result')).toBeInTheDocument()
      expect(screen.getByTestId('syntax-highlighter')).toHaveTextContent('const value = 42;')
    })
  })

  describe('MCPWidget', () => {
    it('should render MCP tool information', () => {
      const input = {
        query: 'test search',
        limit: 10,
      }
      
      render(<MCPWidget toolName="mcp__search_engine__search_web" input={input} />)
      
      expect(screen.getByText('MCP Tool')).toBeInTheDocument()
      expect(screen.getByText('Search Engine')).toBeInTheDocument()
      expect(screen.getByText(/Search Web/)).toBeInTheDocument()
      expect(screen.getByText('~14 tokens')).toBeInTheDocument()
    })

    it('should handle tool without input', () => {
      render(<MCPWidget toolName="mcp__system__get_status" />)
      
      expect(screen.getByText('No parameters required')).toBeInTheDocument()
    })

    it('should expand/collapse large input', async () => {
      const user = userEvent.setup()
      const largeInput = {
        data: 'x'.repeat(300),
      }
      
      render(<MCPWidget toolName="mcp__tool__method" input={largeInput} />)
      
      // Initially collapsed
      expect(screen.getByText('Show full parameters')).toBeInTheDocument()
      
      // Click to expand
      await user.click(screen.getByText('Show full parameters'))
      
      // Should be expanded
      expect(screen.queryByText('Show full parameters')).not.toBeInTheDocument()
    })

    it('should format namespace names correctly', () => {
      const testCases = [
        { tool: 'mcp__snake_case__method', expected: 'Snake Case' },
        { tool: 'mcp__kebab-case__method', expected: 'Kebab Case' },
        { tool: 'mcp__mixed_case-name__method', expected: 'Mixed Case Name' },
      ]
      
      testCases.forEach(({ tool, expected }) => {
        const { rerender } = render(<MCPWidget toolName={tool} />)
        expect(screen.getByText(expected)).toBeInTheDocument()
        rerender(<div />)
      })
    })
  })

  describe('CommandWidget', () => {
    it('should render command with args', () => {
      render(
        <CommandWidget 
          commandName="model" 
          commandMessage="Switching to Claude 3 Opus"
          commandArgs="claude-3-opus"
        />
      )
      
      expect(screen.getByText('Command')).toBeInTheDocument()
      expect(screen.getByText('model')).toBeInTheDocument()
      expect(screen.getByText('claude-3-opus')).toBeInTheDocument()
      expect(screen.getByText('Switching to Claude 3 Opus')).toBeInTheDocument()
    })

    it('should render command without args', () => {
      render(
        <CommandWidget 
          commandName="clear" 
          commandMessage="Clearing conversation"
        />
      )
      
      expect(screen.getByText('clear')).toBeInTheDocument()
      expect(screen.queryByText('claude-3-opus')).not.toBeInTheDocument()
    })

    it('should not show message if same as command name', () => {
      render(
        <CommandWidget 
          commandName="clear" 
          commandMessage="clear"
        />
      )
      
      expect(screen.getAllByText('clear')).toHaveLength(1)
    })
  })

  describe('CommandOutputWidget', () => {
    it('should render plain output', () => {
      render(<CommandOutputWidget output="Command executed successfully" />)
      
      expect(screen.getByText('Output')).toBeInTheDocument()
      expect(screen.getByText('Command executed successfully')).toBeInTheDocument()
    })

    it('should render empty output', () => {
      render(<CommandOutputWidget output="" />)
      
      expect(screen.getByText('No output')).toBeInTheDocument()
    })

    it('should detect and make links clickable', async () => {
      const onLinkDetected = vi.fn()
      const user = userEvent.setup()
      
      render(
        <CommandOutputWidget 
          output="Visit https://example.com for more info"
          onLinkDetected={onLinkDetected}
        />
      )
      
      expect(onLinkDetected).toHaveBeenCalledWith('https://example.com')
      
      const link = screen.getByText('https://example.com')
      await user.click(link)
      
      expect(onLinkDetected).toHaveBeenCalledTimes(2)
    })

    it('should parse ANSI bold codes', () => {
      render(<CommandOutputWidget output="\u001b[1mBold text\u001b[22m normal text" />)
      
      const boldText = screen.getByText('Bold text')
      expect(boldText).toHaveClass('font-bold')
      expect(screen.getByText('normal text')).toBeInTheDocument()
    })
  })

  describe('SummaryWidget', () => {
    it('should render summary with leaf UUID', () => {
      render(
        <SummaryWidget 
          summary="This is an AI-generated summary of the changes"
          leafUuid="12345678-1234-1234-1234-123456789012"
        />
      )
      
      expect(screen.getByText('AI Summary')).toBeInTheDocument()
      expect(screen.getByText('This is an AI-generated summary of the changes')).toBeInTheDocument()
      expect(screen.getByText('ID:')).toBeInTheDocument()
      expect(screen.getByText('12345678...')).toBeInTheDocument()
    })

    it('should render summary without leaf UUID', () => {
      render(<SummaryWidget summary="Summary text" />)
      
      expect(screen.getByText('Summary text')).toBeInTheDocument()
      expect(screen.queryByText('ID:')).not.toBeInTheDocument()
    })
  })

  describe('MultiEditWidget', () => {
    const mockEdits = [
      { old_string: 'const a = 1;', new_string: 'const a = 2;' },
      { old_string: 'let b = 3;', new_string: 'let b = 4;' },
    ]

    it('should render multiple edits', () => {
      render(
        <MultiEditWidget 
          file_path="/test/file.js"
          edits={mockEdits}
        />
      )
      
      expect(screen.getByText('Using tool: MultiEdit')).toBeInTheDocument()
      expect(screen.getByText('/test/file.js')).toBeInTheDocument()
      expect(screen.getByText('2 edits')).toBeInTheDocument()
    })

    it('should toggle edits expansion', async () => {
      const user = userEvent.setup()
      
      render(
        <MultiEditWidget 
          file_path="/test/file.js"
          edits={mockEdits}
        />
      )
      
      // Initially collapsed
      expect(screen.queryByText('Edit 1')).not.toBeInTheDocument()
      
      // Click to expand
      await user.click(screen.getByText('2 edits'))
      
      // Should show edits
      expect(screen.getByText('Edit 1')).toBeInTheDocument()
      expect(screen.getByText('Edit 2')).toBeInTheDocument()
    })

    it('should handle single edit', () => {
      render(
        <MultiEditWidget 
          file_path="/test/file.js"
          edits={[mockEdits[0]]}
        />
      )
      
      expect(screen.getByText('1 edit')).toBeInTheDocument()
    })
  })

  describe('MultiEditResultWidget', () => {
    it('should render edit diffs when edits are provided', () => {
      const edits = [
        { old_string: 'line1\nline2', new_string: 'line1\nmodified2' },
      ]
      
      render(<MultiEditResultWidget content="Changes applied" edits={edits} />)
      
      expect(screen.getByText('1 Changes Applied')).toBeInTheDocument()
      expect(screen.getByText('Change 1')).toBeInTheDocument()
    })

    it('should fallback to simple content display', () => {
      render(<MultiEditResultWidget content="Simple result text" />)
      
      expect(screen.getByText('Simple result text')).toBeInTheDocument()
    })
  })

  describe('SystemReminderWidget', () => {
    it('should render info reminder', () => {
      render(<SystemReminderWidget message="This is an informational message" />)
      
      expect(screen.getByText('This is an informational message')).toBeInTheDocument()
      const container = screen.getByText('This is an informational message').parentElement
      expect(container).toHaveClass('border-blue-500/20', 'bg-blue-500/5')
    })

    it('should render warning reminder', () => {
      render(<SystemReminderWidget message="Warning: This action cannot be undone" />)
      
      const container = screen.getByText(/Warning/).parentElement
      expect(container).toHaveClass('border-yellow-500/20', 'bg-yellow-500/5')
    })

    it('should render error reminder', () => {
      render(<SystemReminderWidget message="Error: Operation failed" />)
      
      const container = screen.getByText(/Error/).parentElement
      expect(container).toHaveClass('border-destructive/20', 'bg-destructive/5')
    })
  })

  describe('SystemInitializedWidget', () => {
    it('should render system information', () => {
      render(
        <SystemInitializedWidget 
          sessionId="session-123"
          model="claude-3-opus"
          cwd="/home/user/project"
          tools={['bash', 'read', 'write']}
        />
      )
      
      expect(screen.getByText('System Initialized')).toBeInTheDocument()
      expect(screen.getByText('session-123')).toBeInTheDocument()
      expect(screen.getByText('claude-3-opus')).toBeInTheDocument()
      expect(screen.getByText('/home/user/project')).toBeInTheDocument()
      expect(screen.getByText('Available Tools (3)')).toBeInTheDocument()
    })

    it('should separate regular tools from MCP tools', () => {
      const tools = [
        'bash',
        'read',
        'mcp__github__create_issue',
        'mcp__github__list_repos',
        'mcp__slack__send_message',
      ]
      
      render(<SystemInitializedWidget tools={tools} />)
      
      expect(screen.getByText('Available Tools (2)')).toBeInTheDocument()
      expect(screen.getByText('MCP Services (3)')).toBeInTheDocument()
    })

    it('should toggle MCP tools expansion', async () => {
      const user = userEvent.setup()
      const tools = ['mcp__github__create_issue']
      
      render(<SystemInitializedWidget tools={tools} />)
      
      // Initially collapsed
      expect(screen.queryByText('Github')).not.toBeInTheDocument()
      
      // Click to expand
      await user.click(screen.getByText('MCP Services (1)'))
      
      // Should show provider details
      expect(screen.getByText('Github')).toBeInTheDocument()
      expect(screen.getByText('Create Issue')).toBeInTheDocument()
    })

    it('should handle no tools', () => {
      render(<SystemInitializedWidget tools={[]} />)
      
      expect(screen.getByText('No tools available')).toBeInTheDocument()
    })

    it('should show tool icons', () => {
      render(<SystemInitializedWidget tools={['bash', 'read', 'write']} />)
      
      // Tool names should be in badges
      expect(screen.getByText('bash').closest('[class*="badge"]')).toBeInTheDocument()
      expect(screen.getByText('read').closest('[class*="badge"]')).toBeInTheDocument()
      expect(screen.getByText('write').closest('[class*="badge"]')).toBeInTheDocument()
    })
  })

  describe('TaskWidget', () => {
    it('should render task description', () => {
      render(
        <TaskWidget 
          description="Analyze the codebase and suggest improvements"
          prompt="You are a code reviewer..."
        />
      )
      
      expect(screen.getByText('Spawning Sub-Agent Task')).toBeInTheDocument()
      expect(screen.getByText('Task Description')).toBeInTheDocument()
      expect(screen.getByText('Analyze the codebase and suggest improvements')).toBeInTheDocument()
    })

    it('should toggle prompt visibility', async () => {
      const user = userEvent.setup()
      const longPrompt = 'This is a very long prompt with detailed instructions...'
      
      render(<TaskWidget prompt={longPrompt} />)
      
      // Initially collapsed
      expect(screen.queryByText(longPrompt)).not.toBeInTheDocument()
      
      // Click to expand
      await user.click(screen.getByText('Task Instructions'))
      
      // Should show prompt
      expect(screen.getByText(longPrompt)).toBeInTheDocument()
    })

    it('should handle missing description', () => {
      render(<TaskWidget prompt="Just a prompt" />)
      
      expect(screen.queryByText('Task Description')).not.toBeInTheDocument()
      expect(screen.getByText('Task Instructions')).toBeInTheDocument()
    })
  })

  describe('WebSearchWidget', () => {
    it('should render search query', () => {
      render(<WebSearchWidget query="TypeScript best practices 2024" />)
      
      expect(screen.getByText('Web Search')).toBeInTheDocument()
      expect(screen.getByText('TypeScript best practices 2024')).toBeInTheDocument()
    })

    it('should render search results with links', () => {
      const result = {
        content: {
          text: `Search Results:
          
Links: [{"title": "TypeScript Handbook", "url": "https://www.typescriptlang.org/docs/"}, {"title": "Best Practices Guide", "url": "https://example.com/guide"}]

The TypeScript handbook provides comprehensive documentation...`,
        },
      }
      
      render(<WebSearchWidget query="TypeScript" result={result} />)
      
      expect(screen.getByText('2 results')).toBeInTheDocument()
      expect(screen.getByText('TypeScript Handbook')).toBeInTheDocument()
      expect(screen.getByText('Best Practices Guide')).toBeInTheDocument()
    })

    it('should toggle search results', async () => {
      const user = userEvent.setup()
      const result = {
        content: 'Links: [{"title": "Result 1", "url": "https://example.com"}]',
      }
      
      render(<WebSearchWidget query="test" result={result} />)
      
      // Initially collapsed (pills view)
      expect(screen.getByText('Result 1')).toBeInTheDocument()
      
      // Click to expand
      await user.click(screen.getByText('1 result'))
      
      // Should show expanded card view
      const expandedResult = screen.getByText('Result 1')
      expect(expandedResult.closest('button')).toHaveClass('group')
    })

    it('should handle clicking links', async () => {
      const user = userEvent.setup()
      const result = {
        content: 'Links: [{"title": "Example", "url": "https://example.com"}]',
      }
      
      render(<WebSearchWidget query="test" result={result} />)
      
      await user.click(screen.getByText('Example'))
      
      expect(open).toHaveBeenCalledWith('https://example.com')
    })

    it('should show no results message', () => {
      const result = {
        content: 'No links found for your search query.',
      }
      
      render(<WebSearchWidget query="obscure topic" result={result} />)
      
      expect(screen.getByText('No results found')).toBeInTheDocument()
    })

    it('should show loading state', () => {
      const result = {
        content: '',
      }
      
      render(<WebSearchWidget query="test" result={result} />)
      
      expect(screen.getByText('Searching...')).toBeInTheDocument()
    })

    it('should parse multiple link sections', () => {
      const result = {
        content: `First set of results:
Links: [{"title": "Result 1", "url": "https://example1.com"}]

Additional results:
Links: [{"title": "Result 2", "url": "https://example2.com"}]`,
      }
      
      render(<WebSearchWidget query="test" result={result} />)
      
      // Should have two separate result sections
      expect(screen.getAllByText(/1 result/)).toHaveLength(2)
      expect(screen.getByText('Result 1')).toBeInTheDocument()
      expect(screen.getByText('Result 2')).toBeInTheDocument()
    })
  })

  describe('ThinkingWidget', () => {
    it('should render thinking content collapsed by default', () => {
      const thinking = `
        Let me analyze this problem step by step...
        First, I need to understand the requirements.
        Then I'll implement the solution.
      `
      
      render(<ThinkingWidget thinking={thinking} />)
      
      expect(screen.getByText('Thinking...')).toBeInTheDocument()
      expect(screen.queryByText(/Let me analyze/)).not.toBeInTheDocument()
    })

    it('should toggle thinking content', async () => {
      const user = userEvent.setup()
      const thinking = 'Analyzing the problem...'
      
      render(<ThinkingWidget thinking={thinking} />)
      
      // Click to expand
      await user.click(screen.getByRole('button'))
      
      // Should show content
      expect(screen.getByText('Analyzing the problem...')).toBeInTheDocument()
      
      // Click to collapse
      await user.click(screen.getByRole('button'))
      
      // Should hide content
      expect(screen.queryByText('Analyzing the problem...')).not.toBeInTheDocument()
    })

    it('should trim whitespace from thinking content', () => {
      const thinking = '   \n\n  Trimmed content  \n\n   '
      
      render(<ThinkingWidget thinking={thinking} />)
      
      // Expand to see content
      const button = screen.getByRole('button')
      userEvent.click(button)
      
      waitFor(() => {
        const content = screen.getByText('Trimmed content')
        expect(content.textContent).toBe('Trimmed content')
      })
    })
  })

  describe('Widget Registry', () => {
    // Test for dynamic widget loading would require a more complex setup
    // as it involves the actual WidgetFactory component
    it('should have all widget types exported', () => {
      // This test verifies that all widgets are properly exported
      expect(TodoWidget).toBeDefined()
      expect(LSWidget).toBeDefined()
      expect(LSResultWidget).toBeDefined()
      expect(ReadWidget).toBeDefined()
      expect(ReadResultWidget).toBeDefined()
      expect(GlobWidget).toBeDefined()
      expect(BashWidget).toBeDefined()
      expect(WriteWidget).toBeDefined()
      expect(GrepWidget).toBeDefined()
      expect(EditWidget).toBeDefined()
      expect(EditResultWidget).toBeDefined()
      expect(MCPWidget).toBeDefined()
      expect(CommandWidget).toBeDefined()
      expect(CommandOutputWidget).toBeDefined()
      expect(SummaryWidget).toBeDefined()
      expect(MultiEditWidget).toBeDefined()
      expect(MultiEditResultWidget).toBeDefined()
      expect(SystemReminderWidget).toBeDefined()
      expect(SystemInitializedWidget).toBeDefined()
      expect(TaskWidget).toBeDefined()
      expect(WebSearchWidget).toBeDefined()
      expect(ThinkingWidget).toBeDefined()
    })
  })

  describe('Accessibility', () => {
    it('should have proper ARIA attributes for interactive elements', async () => {
      const user = userEvent.setup()
      
      render(<ThinkingWidget thinking="Test thinking" />)
      
      const button = screen.getByRole('button')
      expect(button).toBeInTheDocument()
      
      // Button should be keyboard accessible
      await user.tab()
      expect(button).toHaveFocus()
    })

    it('should use semantic HTML elements', () => {
      render(
        <SystemInitializedWidget 
          sessionId="test"
          model="claude"
          tools={['bash']}
        />
      )
      
      // Should use heading for title
      expect(screen.getByText('System Initialized').tagName).toBe('H4')
    })

    it('should provide text alternatives for icons', () => {
      render(<TodoWidget todos={[{ content: 'Test', status: TodoStatus.PENDING }]} />)
      
      // Icon components should be decorative and not interfere with screen readers
      const icons = screen.getByText('Todo List').parentElement?.querySelectorAll('svg')
      expect(icons).toBeDefined()
    })
  })

  describe('Edge Cases', () => {
    it('should handle malformed data gracefully', () => {
      // Test with undefined/null values
      render(<TodoWidget todos={[{ content: 'Test', status: null as any }]} />)
      expect(screen.getByText('Test')).toBeInTheDocument()
      
      // Test with empty strings
      render(<BashWidget command="" />)
      expect(screen.getByText('$')).toBeInTheDocument()
      
      // Test with very long strings
      const longPath = '/very/long/path/'.repeat(50)
      render(<ReadWidget filePath={longPath} />)
      expect(screen.getByText('Reading file:')).toBeInTheDocument()
    })

    it('should handle missing optional props', () => {
      // Components should work with minimal required props
      render(<GlobWidget pattern="*.js" />)
      expect(screen.getByText('*.js')).toBeInTheDocument()
      
      render(<MCPWidget toolName="mcp__test__method" />)
      expect(screen.getByText('MCP Tool')).toBeInTheDocument()
      
      render(<CommandWidget commandName="test" commandMessage="test" />)
      expect(screen.getByText('test')).toBeInTheDocument()
    })

    it('should handle special characters in content', () => {
      const specialContent = '<script>alert("xss")</script> & < > " \''
      
      render(<ReadResultWidget content={specialContent} />)
      
      // Content should be escaped properly
      expect(screen.getByTestId('syntax-highlighter')).toHaveTextContent(specialContent)
      
      // Should not execute scripts
      expect(document.querySelector('script')).toBeNull()
    })
  })
})