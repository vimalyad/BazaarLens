import { AlertTriangle, TrendingDown, TrendingUp, Star, Package } from 'lucide-react';
import type { EventType } from '@/types';

const EVENT_CONFIG: Record<EventType, { icon: typeof AlertTriangle; label: string; color: string }> = {
  price_drop: { icon: TrendingDown, label: 'Competitor Price Drop', color: 'text-destructive' },
  demand_spike: { icon: TrendingUp, label: 'Demand Spike', color: 'text-success' },
  sentiment_crash: { icon: Star, label: 'Sentiment Crash', color: 'text-accent' },
  stock_outage: { icon: Package, label: 'Competitor Stock Outage', color: 'text-primary' },
};

interface EventBannerProps {
  eventType: EventType;
  productName: string;
  triggeredAt: string;
}

export function EventBanner({ eventType, productName, triggeredAt }: EventBannerProps) {
  const config = EVENT_CONFIG[eventType] ?? {
    icon: AlertTriangle,
    label: eventType,
    color: 'text-accent',
  };
  const Icon = config.icon;
  const date = new Date(triggeredAt).toLocaleString('en-IN', {
    dateStyle: 'medium',
    timeStyle: 'short',
  });

  return (
    <div className="border-b border-border bg-card px-4 py-4">
      <div className="flex items-center gap-3">
        <Icon className={`h-6 w-6 flex-shrink-0 ${config.color}`} />
        <div>
          <p className={`text-sm font-bold uppercase tracking-wide ${config.color}`}>
            {config.label}
          </p>
          <p className="truncate text-base font-semibold text-foreground">{productName}</p>
          <p className="text-xs text-muted-foreground">{date}</p>
        </div>
      </div>
    </div>
  );
}
