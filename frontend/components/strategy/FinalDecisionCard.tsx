import { Zap } from 'lucide-react';
import type { FinalDecision } from '@/types';

interface FinalDecisionCardProps {
  decision: FinalDecision;
}

export function FinalDecisionCard({ decision }: FinalDecisionCardProps) {
  return (
    <div className="rounded-xl border-2 border-primary bg-primary/5 p-4">
      <div className="mb-3 flex items-center gap-2">
        <Zap className="h-5 w-5 text-primary" />
        <span className="text-sm font-bold uppercase tracking-wide text-primary">Final Decision</span>
      </div>

      <p className="mb-4 text-lg font-bold text-foreground">{decision.recommended_action}</p>

      <div className="space-y-3 text-sm">
        <div>
          <p className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
            Why this move
          </p>
          <p className="mt-1 text-foreground">{decision.reasoning}</p>
        </div>
        <div>
          <p className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
            Risk if you wait
          </p>
          <p className="mt-1 text-destructive">{decision.risk_assessment}</p>
        </div>
        <div className="rounded-lg bg-accent/10 px-3 py-2">
          <p className="font-semibold text-accent">⏱ {decision.time_sensitivity}</p>
        </div>
      </div>
    </div>
  );
}
