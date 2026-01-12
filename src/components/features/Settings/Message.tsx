import { CheckCircle, WarningCircle } from 'phosphor-react';
import { Text } from '@/components/base/ui/text';
import { ContainerFlex } from '@/components/base/ui';
import { cn } from '@/lib/utils';

export interface MessageProps {
  type: 'success' | 'error';
  text: string;
  className?: string;
}

export function Message({ type, text, className }: MessageProps) {
  return (
    <div
      className={cn(
        'p-4 rounded-lg',
        type === 'success' ? 'bg-secondary' : 'bg-destructive/10',
        className,
      )}
    >
      <ContainerFlex gap="sm" align="start">
        <div className={type === 'success' ? 'text-primary' : 'text-destructive'}>
          {type === 'success' ? <CheckCircle size={20} /> : <WarningCircle size={20} />}
        </div>
        <Text
          size="sm"
          as="span"
          className={type === 'success' ? 'text-foreground' : 'text-destructive'}
        >
          {text}
        </Text>
      </ContainerFlex>
    </div>
  );
}
