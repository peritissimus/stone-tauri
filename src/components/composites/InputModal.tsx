/**
 * InputModal Component - Token-based modal for text input
 *
 * Implements: specs/components.ts#InputModalProps
 * Replaces: Manual Dialog + Input + Button combinations with inline styling
 */

import React, { useState, useEffect, useRef } from 'react';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/base/ui/dialog';
import { Input } from '@/components/base/ui/input';
import { Button } from '@/components/base/ui/button';
import { SizeVariant, sizePaddingClasses } from './tokens';
import { cn } from '@/lib/utils';

export interface InputModalProps {
  /** Modal open state */
  isOpen: boolean;
  /** Close handler */
  onClose: () => void;
  /** Submit handler - receives trimmed input value */
  onSubmit: (value: string) => void;
  /** Size variant for spacing */
  size?: SizeVariant;
  /** Left content (title/header) */
  left?: React.ReactNode;
  /** Right content (additional header elements) */
  right?: React.ReactNode;
  /** Input placeholder */
  placeholder?: string;
  /** Default input value */
  defaultValue?: string;
  /** Submit button label */
  submitLabel?: string;
  /** Cancel button label */
  cancelLabel?: string;
}

/**
 * InputModal composite - consistent modal for text input with token-based sizing.
 *
 * @example
 * <InputModal
 *   isOpen={isOpen}
 *   onClose={onClose}
 *   onSubmit={handleSubmit}
 *   left={<Heading3>Create New Tag</Heading3>}
 *   placeholder="Enter tag name"
 *   submitLabel="Create"
 * />
 */
export const InputModal = React.forwardRef<HTMLDivElement, InputModalProps>(
  (
    {
      isOpen,
      onClose,
      onSubmit,
      size = 'normal',
      left,
      right,
      placeholder = '',
      defaultValue = '',
      submitLabel = 'Create',
      cancelLabel = 'Cancel',
      ...props
    },
    ref,
  ) => {
    const [value, setValue] = useState(defaultValue);
    const inputRef = useRef<HTMLInputElement>(null);

    useEffect(() => {
      if (isOpen) {
        setValue(defaultValue);
        // Focus input when modal opens
        setTimeout(() => {
          inputRef.current?.focus();
        }, 100);
      }
    }, [isOpen, defaultValue]);

    const handleSubmit = (e: React.FormEvent) => {
      e.preventDefault();
      const trimmedValue = value.trim();
      if (trimmedValue) {
        onSubmit(trimmedValue);
        setValue('');
      }
    };

    const handleKeyDown = (e: React.KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    };

    return (
      <Dialog open={isOpen} onOpenChange={onClose}>
        <DialogContent ref={ref} className="sm:max-w-[425px]" {...props}>
          <DialogHeader>
            <div className="flex items-center justify-between">
              <DialogTitle className={cn("flex-1", !left && "sr-only")}>
                {left || "Input Modal"}
              </DialogTitle>
              {right && <div className="flex items-center gap-2">{right}</div>}
            </div>
          </DialogHeader>
          <form onSubmit={handleSubmit}>
            <div className={cn('py-4', sizePaddingClasses[size])}>
              <Input
                ref={inputRef}
                type="text"
                value={value}
                onChange={(e) => setValue(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder={placeholder}
                autoFocus
              />
            </div>
            <DialogFooter>
              <Button type="button" variant="outline" onClick={onClose}>
                {cancelLabel}
              </Button>
              <Button type="submit" disabled={!value.trim()}>
                {submitLabel}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    );
  },
);

InputModal.displayName = 'InputModal';
