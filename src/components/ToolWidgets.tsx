/**
 * Transitional file to help migrate from the old ToolWidgets.tsx to the new widgets folder structure.
 * 
 * This file provides backward compatibility while applications migrate to the new structure.
 * 
 * Migration Guide:
 * 1. Update imports from './ToolWidgets' to './widgets'
 * 2. Use the WidgetFactory for dynamic widget selection
 * 3. Import specific widgets directly when needed
 * 
 * @deprecated This file will be removed in the next major version. Use ./widgets directly.
 */

// Re-export everything from the new widgets folder for backward compatibility
export * from "./widgets";