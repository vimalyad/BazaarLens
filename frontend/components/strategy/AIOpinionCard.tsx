import { cn } from '@/lib/utils';
import type { AIOpinion, Urgency } from '@/types';

const URGENCY_STYLES: Record<Urgency, { border: string; badge: string }> = {
  high: { border: 'border-destructive/50', badge: 'bg-destructive/10 text-destructive' },
  medium: { border: 'border-accent/50', badge: 'bg-accent/10 text-accent' },
  low: { border: 'border-success/50', badge: 'bg-success/10 text-success' },
};

interface AIOpinionCardProps {
  role: string;
  emoji: string;
  opinion: AIOpinion;
  className?: string;
}

export function AIOpinionCard({ role, emoji, opinion, className }: AIOpinionCardProps) {
  const urgency = (opinion.urgency as Urgency) in URGENCY_STYLES
    ? (opinion.urgency as Urgency)
    : 'medium';
  const styles = URGENCY_STYLES[urgency];

  return (
    <div className={cn('rounded-xl border bg-card p-4', styles.border, className)}>
      <div className="mb-3 flex items-center justify-between">
        <span className="text-sm font-bold text-foreground">
          {emoji} {role}
        </span>
        <span className={cn('rounded-full px-2 py-0.5 text-xs font-semibold capitalize', styles.badge)}>
          {urgency}
        </span>
      </div>
      <p className="mb-3 text-sm text-muted-foreground">{opinion.opinion}</p>
      <div className="rounded-lg bg-muted/30 p-3">
        <p className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">Action</p>
        <p className="mt-1 text-sm font-medium text-foreground">{opinion.action}</p>
      </div>
    </div>
  );
}
