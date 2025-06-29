import React, { useEffect, useState, useRef } from "react";
import { Calendar, Clock } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Popover } from "@/components/ui/popover";
import { cn } from "@/lib/utils";

interface DateTimePickerProps {
  /**
   * The selected datetime value (ISO 8601 string)
   */
  value?: string;
  /**
   * Callback when datetime changes
   */
  onChange: (value: string | undefined) => void;
  /**
   * Optional placeholder text
   */
  placeholder?: string;
  /**
   * Optional className
   */
  className?: string;
  /**
   * Optional disabled state
   */
  disabled?: boolean;
  /**
   * Minimum datetime (ISO 8601 string)
   */
  min?: string;
}

/**
 * DateTimePicker component for selecting date and time
 * 
 * @example
 * <DateTimePicker
 *   value={scheduledTime}
 *   onChange={setScheduledTime}
 *   placeholder="Select date and time"
 * />
 */
export const DateTimePicker: React.FC<DateTimePickerProps> = ({
  value,
  onChange,
  placeholder = "Select date and time",
  className,
  disabled = false,
  min,
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [tempValue, setTempValue] = useState<string>("");
  const inputRef = useRef<HTMLInputElement>(null);
  
  // Parse the ISO string to local datetime-local format
  const toLocalDateTimeString = (isoString?: string) => {
    if (!isoString) return "";
    const date = new Date(isoString);
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    const hours = String(date.getHours()).padStart(2, '0');
    const minutes = String(date.getMinutes()).padStart(2, '0');
    return `${year}-${month}-${day}T${hours}:${minutes}`;
  };

  // Convert local datetime-local format to ISO string
  const toISOString = (localDateTime: string) => {
    if (!localDateTime) return undefined;
    const date = new Date(localDateTime);
    return date.toISOString();
  };

  // Format datetime for display
  const formatDateTime = (isoString?: string) => {
    if (!isoString) return "";
    const date = new Date(isoString);
    const options: Intl.DateTimeFormatOptions = {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      hour12: true
    };
    return date.toLocaleString(undefined, options);
  };

  const [localValue, setLocalValue] = useState(toLocalDateTimeString(value));

  useEffect(() => {
    setLocalValue(toLocalDateTimeString(value));
  }, [value]);

  // Initialize temp value when popover opens
  useEffect(() => {
    if (isOpen) {
      setTempValue(localValue);
    }
  }, [isOpen, localValue]);

  const handleDateTimeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = e.target.value;
    setTempValue(newValue);
  };

  const handleSave = () => {
    setLocalValue(tempValue);
    onChange(toISOString(tempValue));
    setIsOpen(false);
  };

  const handleClear = () => {
    setLocalValue("");
    setTempValue("");
    onChange(undefined);
    setIsOpen(false);
  };

  const handleCancel = () => {
    setTempValue(localValue);
    setIsOpen(false);
  };

  const minDateTime = min ? toLocalDateTimeString(min) : toLocalDateTimeString(new Date().toISOString());

  const triggerButton = (
    <Button
      variant="outline"
      className={cn(
        "justify-start text-left font-normal",
        !value && "text-muted-foreground",
        className
      )}
      disabled={disabled}
    >
      <Calendar className="mr-2 h-4 w-4" />
      {value ? formatDateTime(value) : placeholder}
    </Button>
  );

  const popoverContent = (
    <div className="space-y-4 bg-background border border-border rounded-lg shadow-lg p-4">
      <div className="space-y-2">
        <Label htmlFor="datetime-input" className="text-sm font-medium text-foreground">
          Select Date & Time
        </Label>
        <div className="relative">
          <Input
            ref={inputRef}
            id="datetime-input"
            type="datetime-local"
            value={tempValue}
            onChange={handleDateTimeChange}
            min={minDateTime}
            className="w-full pl-10 bg-background text-foreground border-input focus:border-ring"
            style={{
              colorScheme: 'dark'
            }}
          />
          <Clock className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground pointer-events-none" />
        </div>
      </div>
      
      <div className="flex items-center justify-between space-x-2">
        <Button
          variant="outline"
          size="sm"
          onClick={handleCancel}
          className="flex-1"
        >
          Cancel
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={handleClear}
          className="flex-1"
          disabled={!tempValue}
        >
          Clear
        </Button>
        <Button
          variant="default"
          size="sm"
          onClick={handleSave}
          className="flex-1"
          disabled={!tempValue}
        >
          Save
        </Button>
      </div>
      
      {tempValue && (
        <div className="text-xs text-muted-foreground border-t border-border pt-3">
          <p>Selected: {formatDateTime(toISOString(tempValue))}</p>
          <p className="mt-1">Timezone: {Intl.DateTimeFormat().resolvedOptions().timeZone}</p>
        </div>
      )}
    </div>
  );

  return (
    <Popover 
      trigger={triggerButton}
      content={popoverContent}
      open={isOpen}
      onOpenChange={setIsOpen}
      className="w-auto p-0"
      align="start"
    />
  );
};