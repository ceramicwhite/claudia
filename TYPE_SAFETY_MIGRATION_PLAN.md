# TypeScript Strict Mode Migration Plan

## Overview
This document outlines the strategy for fixing TypeScript type issues after enabling strict mode.

## Current Issues Summary

### 1. Service Layer Issues (High Priority)
- **BaseService.invoke() signature mismatch**: Subclasses calling with 2 params instead of 3
- **Missing schemas**: Services need Zod schemas for type validation
- **Type inference**: Services returning `unknown` instead of proper types

### 2. Widget Factory Issues (Medium Priority)
- **Type incompatibility**: Widget components expect specific props but receive BaseWidgetProps
- **Missing type unions**: Need discriminated unions for widget props

### 3. Project Service Issues (Medium Priority)
- **Optional properties**: `tags` and `metadata` marked as optional but expected as required
- **Type mismatches**: Various property type incompatibilities

### 4. Any Type Usage (Low Priority)
- **39 files** contain `any` types that need replacement
- Focus on event handlers and API responses first

## Migration Strategy

### Phase 1: Fix Critical Service Layer (Immediate)
1. Update all service methods to use proper invoke signatures
2. Create Zod schemas for all API responses
3. Fix the invoke method calls to include schemas

### Phase 2: Fix Widget System (Next)
1. Create discriminated union types for widget props
2. Update WidgetFactory to handle type narrowing
3. Fix individual widget components

### Phase 3: Fix Project Service (Then)
1. Update type definitions to handle optional properties
2. Add proper defaults for optional fields
3. Fix validation schemas

### Phase 4: Remove Any Types (Finally)
1. Replace `any` with `unknown` and add type guards
2. Add proper event handler types
3. Fix remaining type assertions

## Implementation Order

1. **Fix BaseService subclasses** - Start with simpler services:
   - session.service.ts
   - usage.service.ts
   - sandbox.service.ts
   - mcp.service.ts
   - agent.service.ts

2. **Fix Widget System**
   - Update widget types
   - Fix WidgetFactory

3. **Fix Project Service**
   - Update type definitions
   - Fix optional property handling

4. **Clean up any types**
   - Focus on critical paths first
   - Add proper type guards

## Type Safety Guidelines

1. **Never use `any`** - Use `unknown` with type guards instead
2. **Always validate external data** - Use Zod schemas for API responses
3. **Prefer strict types** - Avoid optional properties unless truly optional
4. **Use discriminated unions** - For variant types like widgets
5. **Add explicit return types** - For all functions and methods