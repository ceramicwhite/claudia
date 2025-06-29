import React, { useEffect, useState, useMemo, useRef } from "react";
import { Calendar, ChevronDown } from "lucide-react";
import { Button } from "@/components/ui/button";
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

interface DateTimeComponents {
  year: number;
  month: number;
  day: number;
  hour: number;
  minute: number;
  isPM: boolean;
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
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [dateTimeComponents, setDateTimeComponents] = useState<DateTimeComponents>(() => {
    const now = new Date();
    return {
      year: now.getFullYear(),
      month: now.getMonth() + 1,
      day: now.getDate(),
      hour: now.getHours() % 12 || 12,
      minute: Math.ceil(now.getMinutes() / 5) * 5,
      isPM: now.getHours() >= 12
    };
  });

  // Calculate days in month
  const daysInMonth = useMemo(() => {
    return new Date(dateTimeComponents.year, dateTimeComponents.month, 0).getDate();
  }, [dateTimeComponents.year, dateTimeComponents.month]);

  // Generate arrays for dropdowns
  const months = Array.from({ length: 12 }, (_, i) => ({
    value: i + 1,
    label: new Date(2000, i, 1).toLocaleString('default', { month: 'long' })
  }));

  const days = Array.from({ length: daysInMonth }, (_, i) => i + 1);
  const hours = Array.from({ length: 12 }, (_, i) => i + 1);
  const minutes = Array.from({ length: 12 }, (_, i) => i * 5);

  // Convert components to Date object
  const componentsToDate = (components: DateTimeComponents): Date => {
    const { year, month, day, hour, minute, isPM } = components;
    let hour24 = hour;
    
    if (isPM && hour !== 12) {
      hour24 = hour + 12;
    } else if (!isPM && hour === 12) {
      hour24 = 0;
    }
    
    return new Date(year, month - 1, day, hour24, minute);
  };

  // Parse ISO string to components
  const parseISOToComponents = (isoString?: string): DateTimeComponents => {
    if (!isoString) {
      const now = new Date();
      return {
        year: now.getFullYear(),
        month: now.getMonth() + 1,
        day: now.getDate(),
        hour: now.getHours() % 12 || 12,
        minute: Math.ceil(now.getMinutes() / 5) * 5,
        isPM: now.getHours() >= 12
      };
    }
    
    const date = new Date(isoString);
    return {
      year: date.getFullYear(),
      month: date.getMonth() + 1,
      day: date.getDate(),
      hour: date.getHours() % 12 || 12,
      minute: date.getMinutes(),
      isPM: date.getHours() >= 12
    };
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

  // Initialize from value prop
  useEffect(() => {
    if (value) {
      setDateTimeComponents(parseISOToComponents(value));
    }
  }, [value]);

  // Update parent when components change
  const updateDateTime = (newComponents: Partial<DateTimeComponents>) => {
    const updated = { ...dateTimeComponents, ...newComponents };
    
    // Auto-adjust year if month or day changed
    if ('month' in newComponents || 'day' in newComponents) {
      updated.year = autoAdjustYear(updated.month, updated.day);
    }
    
    // Ensure day is valid for the selected month
    const maxDay = new Date(updated.year, updated.month, 0).getDate();
    if (updated.day > maxDay) {
      updated.day = maxDay;
    }
    
    setDateTimeComponents(updated);
    
    const date = componentsToDate(updated);
    onChange(date.toISOString());
  };

  // Initialize when popover opens
  useEffect(() => {
    if (isOpen && !value) {
      const date = componentsToDate(dateTimeComponents);
      onChange(date.toISOString());
    }
  }, [isOpen]);

  // Auto-set year based on selected month
  const autoAdjustYear = (month: number, day: number) => {
    const now = new Date();
    const currentMonth = now.getMonth() + 1;
    const currentYear = now.getFullYear();
    
    // If selecting a month that's earlier than current month, assume next year
    if (month < currentMonth || (month === currentMonth && day < now.getDate())) {
      return currentYear + 1;
    }
    return currentYear;
  };

  const handleClear = () => {
    onChange(undefined);
    setIsOpen(false);
  };


  const DropdownMenu: React.FC<{
    items: Array<{ value: number | string; label: string }>;
    selectedValue: number | string;
    onSelect: (value: number | string) => void;
  }> = ({ items, selectedValue, onSelect }) => {
    const [isOpen, setIsOpen] = useState(false);
    const dropdownRef = useRef<HTMLDivElement>(null);
    const buttonRef = useRef<HTMLButtonElement>(null);
    
    // Close on click outside
    useEffect(() => {
      if (!isOpen) return;
      
      const handleClickOutside = (event: MouseEvent) => {
        if (
          dropdownRef.current &&
          buttonRef.current &&
          !dropdownRef.current.contains(event.target as Node) &&
          !buttonRef.current.contains(event.target as Node)
        ) {
          setIsOpen(false);
        }
      };
      
      document.addEventListener("mousedown", handleClickOutside);
      return () => document.removeEventListener("mousedown", handleClickOutside);
    }, [isOpen]);
    
    return (
      <div className="relative">
        <button
          ref={buttonRef}
          type="button"
          onClick={() => setIsOpen(!isOpen)}
          className="w-full flex items-center justify-between px-3 py-2 text-sm bg-background border border-input rounded-md hover:bg-accent hover:text-accent-foreground transition-colors"
        >
          <span>{items.find(item => item.value === selectedValue)?.label || selectedValue}</span>
          <ChevronDown className={cn("ml-2 h-4 w-4 opacity-50 transition-transform", isOpen && "rotate-180")} />
        </button>
        
        {isOpen && (
          <div 
            ref={dropdownRef}
            className="absolute z-[100] w-full mt-1 bg-popover rounded-md shadow-lg border border-border animate-in fade-in-0 zoom-in-95"
            style={{
              maxHeight: 'min(300px, calc(100vh - 200px))',
              overflowY: 'auto'
            }}
          >
            <div className="p-1">
              {items.map(item => (
                <button
                  key={item.value}
                  type="button"
                  onClick={() => {
                    onSelect(item.value);
                    setIsOpen(false);
                  }}
                  className={cn(
                    "w-full px-3 py-1.5 text-sm text-left rounded-sm hover:bg-accent hover:text-accent-foreground transition-colors",
                    selectedValue === item.value && "bg-accent text-accent-foreground"
                  )}
                >
                  {item.label}
                </button>
              ))}
            </div>
          </div>
        )}
      </div>
    );
  };

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
    <div className="w-[320px] space-y-4 bg-background border border-border rounded-lg shadow-lg p-4 overflow-visible">
      <div className="space-y-3 overflow-visible">
        <Label className="text-sm font-medium text-foreground">
          Select Date & Time
        </Label>
        
        {/* Date Selection */}
        <div className="space-y-2">
          <Label className="text-xs text-muted-foreground">Date</Label>
          <div className="grid grid-cols-2 gap-2">
            <DropdownMenu
              items={months}
              selectedValue={dateTimeComponents.month}
              onSelect={(value) => updateDateTime({ month: value as number })}
            />
            
            <DropdownMenu
              items={days.map(d => ({ value: d, label: d.toString() }))}
              selectedValue={dateTimeComponents.day}
              onSelect={(value) => updateDateTime({ day: value as number })}
            />
          </div>
          <p className="text-xs text-muted-foreground text-center mt-1">
            Year: {dateTimeComponents.year}
          </p>
        </div>

        {/* Time Selection */}
        <div className="space-y-2">
          <Label className="text-xs text-muted-foreground">Time</Label>
          <div className="grid grid-cols-3 gap-2">
            <DropdownMenu
              items={hours.map(h => ({ value: h, label: h.toString() }))}
              selectedValue={dateTimeComponents.hour}
              onSelect={(value) => updateDateTime({ hour: value as number })}
            />
            
            <DropdownMenu
              items={minutes.map(m => ({ 
                value: m, 
                label: m.toString().padStart(2, '0') 
              }))}
              selectedValue={dateTimeComponents.minute}
              onSelect={(value) => updateDateTime({ minute: value as number })}
            />
            
            <DropdownMenu
              items={[
                { value: 'AM', label: 'AM' },
                { value: 'PM', label: 'PM' }
              ]}
              selectedValue={dateTimeComponents.isPM ? 'PM' : 'AM'}
              onSelect={(value) => updateDateTime({ isPM: value === 'PM' })}
            />
          </div>
        </div>

        <p className="text-xs text-muted-foreground">
          Changes are saved automatically
        </p>
      </div>
      
      <div className="flex items-center justify-center">
        <Button
          variant="outline"
          size="sm"
          onClick={handleClear}
          disabled={!value}
        >
          Clear Schedule
        </Button>
      </div>
      
      {value && (
        <div className="text-xs text-muted-foreground border-t border-border pt-3">
          <p>Scheduled for: {formatDateTime(value)}</p>
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
      className="w-auto p-0 z-50"
      align="start"
    />
  );
};