# Refactoring Best Practices and Migration Strategies (2025)

## Executive Summary

This document compiles research on current best practices for refactoring TypeScript/React applications, focusing on identified needs in the Claudia codebase. The research covers TypeScript strict typing, React performance optimization, Tauri architecture patterns, error handling with Zod, and modern state management approaches.

## 1. TypeScript Strict Typing and Type Safety Patterns

### Key Migration Strategies

#### 1.1 Enabling Strict Mode
```typescript
// tsconfig.json
{
  "compilerOptions": {
    "strict": true,  // Mandatory for all TypeScript projects
    "strictNullChecks": true,
    "strictFunctionTypes": true,
    "strictBindCallApply": true,
    "strictPropertyInitialization": true,
    "noImplicitAny": true,
    "noImplicitThis": true,
    "useUnknownInCatchVariables": true
  }
}
```

#### 1.2 Migrating from `any` Types

**Pattern 1: Using `unknown` for safer type assertions**
```typescript
// Before
function processData(data: any) {
  return data.someProperty;
}

// After
function processData(data: unknown) {
  if (typeof data === 'object' && data !== null && 'someProperty' in data) {
    return (data as { someProperty: unknown }).someProperty;
  }
  throw new Error('Invalid data structure');
}
```

**Pattern 2: Type guards for runtime validation**
```typescript
function isValidData(value: unknown): value is { someProperty: string } {
  return (
    typeof value === 'object' &&
    value !== null &&
    'someProperty' in value &&
    typeof (value as any).someProperty === 'string'
  );
}
```

#### 1.3 Generic Constraints
```typescript
// Constraining generic types for better type safety
function makeSchemaOptional<T extends z.ZodType<string>>(schema: T) {
  return schema.optional();
}

// Using ZodTypeAny for flexible but type-safe schemas
function inferSchema<T extends z.ZodTypeAny>(schema: T) {
  return schema;
}
```

### Breaking Changes to Watch For

1. **Strict Function Types**: Functions with contravariant parameters will fail type checks
2. **Unknown in Catch**: Catch variables default to `unknown` instead of `any`
3. **Exact Optional Properties**: Optional properties can't be assigned `undefined` directly

## 2. React Performance Optimization Patterns

### 2.1 Memoization Best Practices

**useMemo for Expensive Calculations**
```typescript
const visibleTodos = useMemo(
  () => filterTodos(todos, tab),
  [todos, tab] // Dependencies - only re-calculate when these change
);
```

**useCallback for Stable Function References**
```typescript
const handleSubmit = useCallback((orderDetails) => {
  post('/product/' + productId + '/buy', {
    referrer,
    orderDetails,
  });
}, [productId, referrer]); // Dependencies for function recreation
```

**React.memo for Component Memoization**
```typescript
const ShippingForm = memo(function ShippingForm({ onSubmit }) {
  // Component only re-renders if props change
  return <form>...</form>;
});
```

### 2.2 Context Optimization Pattern
```typescript
// Optimize context values to prevent unnecessary re-renders
function MyApp() {
  const [currentUser, setCurrentUser] = useState(null);

  const login = useCallback((response) => {
    storeCredentials(response.credentials);
    setCurrentUser(response.user);
  }, []);

  const contextValue = useMemo(() => ({
    currentUser,
    login
  }), [currentUser, login]);

  return (
    <AuthContext.Provider value={contextValue}>
      <Page />
    </AuthContext.Provider>
  );
}
```

### 2.3 React 19 Resource Preloading
```typescript
import { prefetchDNS, preconnect, preload, preinit } from 'react-dom'

function MyComponent() {
  preinit('https://.../script.js', { as: 'script' }); // Load and execute eagerly
  preload('https://.../font.woff', { as: 'font' }); // Preload resource
  prefetchDNS('https://...'); // DNS prefetch for potential requests
  preconnect('https://...'); // Establish early connection
}
```

## 3. Tauri Application Architecture Best Practices

### 3.1 State Management Pattern
```rust
use std::sync::Mutex;
use tauri::{Builder, Manager};

#[derive(Default)]
struct AppState {
  counter: u32,
}

// Type alias for consistency
type AppStateMutex = Mutex<AppState>;

fn main() {
  Builder::default()
    .setup(|app| {
      app.manage(Mutex::new(AppState::default()));
      Ok(())
    })
    .run(tauri::generate_context!())
    .unwrap();
}
```

### 3.2 Command Error Handling
```rust
#[derive(Debug, thiserror::Error)]
enum Error {
  #[error(transparent)]
  Io(#[from] std::io::Error),
  #[error("failed to parse: {0}")]
  Parse(String),
}

// Structured error serialization
#[derive(serde::Serialize)]
#[serde(tag = "kind", content = "message")]
enum ErrorKind {
  Io(String),
  Parse(String),
}

impl serde::Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::ser::Serializer,
  {
    let error_message = self.to_string();
    let error_kind = match self {
      Self::Io(_) => ErrorKind::Io(error_message),
      Self::Parse(_) => ErrorKind::Parse(error_message),
    };
    error_kind.serialize(serializer)
  }
}
```

### 3.3 Structured Capability System
```
/src-tauri
  /capabilities
    filesystem.json
    dialog.json
  /permissions
    custom-permission.toml
  tauri.conf.json
```

## 4. Error Handling with Zod

### 4.1 Safe Parsing Pattern
```typescript
// Use safeParse instead of parse to avoid throwing
const result = schema.safeParse(input);
if (!result.success) {
  // Access detailed error information
  result.error.issues.forEach(issue => {
    console.log(`${issue.path}: ${issue.message}`);
  });
} else {
  // Use validated data
  const data = result.data;
}
```

### 4.2 Custom Error Messages (Zod v4)
```typescript
// New unified error parameter
z.string({ 
  error: (issue) => {
    if (issue.input === undefined) return "This field is required";
    return "Invalid input";
  }
});

// Validation with refinements
const passwordForm = z
  .object({
    password: z.string(),
    confirm: z.string(),
  })
  .refine((data) => data.password === data.confirm, {
    message: "Passwords don't match",
    path: ["confirm"], // Error location
  });
```

### 4.3 Transform with Validation
```typescript
const numberInString = z.string().transform((val, ctx) => {
  const parsed = parseInt(val);
  if (isNaN(parsed)) {
    ctx.addIssue({
      code: z.ZodIssueCode.custom,
      message: "Not a number",
    });
    return z.NEVER; // Early return with type safety
  }
  return parsed;
});
```

## 5. Modern State Management with Zustand

### 5.1 TypeScript-First Setup
```typescript
import { create } from 'zustand'
import { devtools, persist } from 'zustand/middleware'

interface BearState {
  bears: number
  increase: (by: number) => void
}

const useBearStore = create<BearState>()(
  devtools(
    persist(
      (set) => ({
        bears: 0,
        increase: (by) => set((state) => ({ bears: state.bears + by })),
      }),
      {
        name: 'bear-storage',
      },
    ),
  ),
)
```

### 5.2 Slices Pattern for Modular Stores
```typescript
import { create, StateCreator } from 'zustand'

interface BearSlice {
  bears: number
  addBear: () => void
}

interface FishSlice {
  fishes: number
  addFish: () => void
}

const createBearSlice: StateCreator<
  BearSlice & FishSlice,
  [],
  [],
  BearSlice
> = (set) => ({
  bears: 0,
  addBear: () => set((state) => ({ bears: state.bears + 1 })),
})

const createFishSlice: StateCreator<
  BearSlice & FishSlice,
  [],
  [],
  FishSlice
> = (set) => ({
  fishes: 0,
  addFish: () => set((state) => ({ fishes: state.fishes + 1 })),
})

const useBoundStore = create<BearSlice & FishSlice>()((...a) => ({
  ...createBearSlice(...a),
  ...createFishSlice(...a),
}))
```

### 5.3 Performance Optimization with Selectors
```typescript
import { useShallow } from 'zustand/react/shallow'

// Prevent unnecessary re-renders with shallow comparison
const { nuts, honey } = useBearStore(
  useShallow((state) => ({ nuts: state.nuts, honey: state.honey })),
)

// Auto-generating selectors
const createSelectors = <S extends UseBoundStore<StoreApi<object>>>(
  _store: S,
) => {
  const store = _store as WithSelectors<typeof _store>
  store.use = {}
  for (const k of Object.keys(store.getState())) {
    ;(store.use as any)[k] = () => store((s) => s[k as keyof typeof s])
  }
  return store
}
```

## 6. Event System Patterns

### 6.1 Type-Safe Event Emitters
```typescript
// Using Zod for event payload validation
const EventPayloadSchema = z.object({
  type: z.enum(['started', 'progress', 'finished']),
  data: z.unknown(),
});

type EventPayload = z.infer<typeof EventPayloadSchema>;

class TypedEventEmitter extends EventEmitter {
  emit(event: string, payload: unknown) {
    const validated = EventPayloadSchema.parse(payload);
    return super.emit(event, validated);
  }
}
```

### 6.2 Tauri Event System
```rust
// Structured event payloads
#[derive(Clone, Serialize)]
#[serde(tag = "event", content = "data")]
pub enum DownloadEvent {
    #[serde(rename_all = "camelCase")]
    Started { content_length: Option<u64> },
    #[serde(rename_all = "camelCase")]
    Progress { chunk_length: usize },
    Finished,
}

// Type-safe event emission
app.emit("download-started", DownloadStarted {
    url: &url,
    download_id,
    content_length
}).unwrap();
```

## 7. Migration Checklist

### Phase 1: Foundation
- [ ] Enable TypeScript strict mode
- [ ] Set up ESLint with @typescript-eslint
- [ ] Configure Prettier for consistent formatting
- [ ] Implement pre-commit hooks for type checking

### Phase 2: Type Safety
- [ ] Replace all `any` types with `unknown` or specific types
- [ ] Implement runtime validation with Zod schemas
- [ ] Add type guards for external data
- [ ] Create branded types for domain primitives

### Phase 3: React Optimization
- [ ] Audit and add React.memo to expensive components
- [ ] Implement useMemo for expensive calculations
- [ ] Add useCallback for stable function references
- [ ] Optimize context providers with memoization

### Phase 4: State Management
- [ ] Migrate to Zustand with TypeScript
- [ ] Implement slices pattern for modular state
- [ ] Add middleware for persistence and devtools
- [ ] Create auto-generating selectors

### Phase 5: Error Handling
- [ ] Implement Zod schemas for all external data
- [ ] Create structured error types
- [ ] Add error boundaries in React
- [ ] Implement proper error serialization in Tauri

## 8. Tools and Resources

### Recommended Tools
- **TypeScript**: v5.x with strict mode
- **Zod**: v3.x for runtime validation
- **Zustand**: v4.x for state management
- **React**: v18.x with concurrent features
- **Tauri**: v2.x for desktop applications

### Useful Commands
```bash
# Type checking
tsc --noEmit

# Find any types
grep -r "any" src/ --include="*.ts" --include="*.tsx"

# Audit dependencies
npm audit
cargo audit

# Bundle analysis
npm run build -- --analyze
```

## Conclusion

This research provides a comprehensive guide for modernizing the Claudia codebase with current best practices. The focus should be on incremental migration, starting with enabling strict TypeScript mode and gradually replacing unsafe patterns with type-safe alternatives. Each phase builds upon the previous one, ensuring a stable and maintainable codebase throughout the migration process.