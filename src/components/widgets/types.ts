/**
 * Widget Type Definitions
 * 
 * This file defines all widget types using discriminated unions
 * to ensure type safety across the widget system.
 */

// Base widget props that all widgets share
export interface BaseWidgetProps {
  toolName: string;
  params?: Record<string, unknown>;
  result?: unknown;
}

// Todo Widget Props
export interface TodoWidgetProps extends BaseWidgetProps {
  toolName: 'TodoWrite' | 'TodoRead';
  todos: Array<{
    id: string;
    content: string;
    status: string;
    priority: string;
  }>;
}

// File Operation Widget Props
export interface LSWidgetProps extends BaseWidgetProps {
  toolName: 'LS';
  path: string;
}

export interface LSResultWidgetProps extends BaseWidgetProps {
  toolName: 'LSResult';
  content: string[];
}

export interface ReadWidgetProps extends BaseWidgetProps {
  toolName: 'Read';
  filePath: string;
}

export interface ReadResultWidgetProps extends BaseWidgetProps {
  toolName: 'ReadResult';
  content: string;
}

export interface WriteWidgetProps extends BaseWidgetProps {
  toolName: 'Write';
  filePath: string;
  content: string;
}

export interface EditWidgetProps extends BaseWidgetProps {
  toolName: 'Edit';
  file_path: string;
  old_string: string;
  new_string: string;
}

export interface EditResultWidgetProps extends BaseWidgetProps {
  toolName: 'EditResult';
  content: string;
}

export interface MultiEditWidgetProps extends BaseWidgetProps {
  toolName: 'MultiEdit';
  file_path: string;
  edits: Array<{
    old_string: string;
    new_string: string;
  }>;
}

// Search Widget Props
export interface GrepWidgetProps extends BaseWidgetProps {
  toolName: 'Grep';
  pattern: string;
  path?: string;
  include?: string;
}

export interface GlobWidgetProps extends BaseWidgetProps {
  toolName: 'Glob';
  pattern: string;
  path?: string;
}

// Command Widget Props
export interface BashWidgetProps extends BaseWidgetProps {
  toolName: 'Bash';
  command: string;
  description?: string;
}

// System Widget Props
export interface SystemReminderWidgetProps extends BaseWidgetProps {
  toolName: 'SystemReminder';
  message: string;
}

// Union type of all widget props
export type WidgetProps = 
  | TodoWidgetProps
  | LSWidgetProps
  | LSResultWidgetProps
  | ReadWidgetProps
  | ReadResultWidgetProps
  | WriteWidgetProps
  | EditWidgetProps
  | EditResultWidgetProps
  | MultiEditWidgetProps
  | GrepWidgetProps
  | GlobWidgetProps
  | BashWidgetProps
  | SystemReminderWidgetProps;

// Type guard functions
export function isTodoWidget(props: BaseWidgetProps): props is TodoWidgetProps {
  return props.toolName === 'TodoWrite' || props.toolName === 'TodoRead';
}

export function isLSWidget(props: BaseWidgetProps): props is LSWidgetProps {
  return props.toolName === 'LS';
}

export function isReadWidget(props: BaseWidgetProps): props is ReadWidgetProps {
  return props.toolName === 'Read';
}

export function isWriteWidget(props: BaseWidgetProps): props is WriteWidgetProps {
  return props.toolName === 'Write';
}

export function isEditWidget(props: BaseWidgetProps): props is EditWidgetProps {
  return props.toolName === 'Edit';
}

export function isMultiEditWidget(props: BaseWidgetProps): props is MultiEditWidgetProps {
  return props.toolName === 'MultiEdit';
}

export function isGrepWidget(props: BaseWidgetProps): props is GrepWidgetProps {
  return props.toolName === 'Grep';
}

export function isGlobWidget(props: BaseWidgetProps): props is GlobWidgetProps {
  return props.toolName === 'Glob';
}

export function isBashWidget(props: BaseWidgetProps): props is BashWidgetProps {
  return props.toolName === 'Bash';
}

export function isSystemReminderWidget(props: BaseWidgetProps): props is SystemReminderWidgetProps {
  return props.toolName === 'SystemReminder';
}