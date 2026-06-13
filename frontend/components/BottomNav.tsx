'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { Camera, Eye } from 'lucide-react';
import { cn } from '@/lib/utils';

const TABS = [
  { href: '/scan', label: 'Scan', icon: Camera },
  { href: '/watchlist', label: 'Watchlist', icon: Eye },
] as const;

export function BottomNav() {
  const pathname = usePathname();

  return (
    <nav className="fixed bottom-0 left-0 right-0 z-50 border-t border-border bg-card">
      <ul className="flex">
        {TABS.map(({ href, label, icon: Icon }) => {
          const active = pathname === href || pathname.startsWith(href + '/');
          return (
            <li key={href} className="flex-1">
              <Link
                href={href}
                className={cn(
                  'flex min-h-[56px] flex-col items-center justify-center gap-1 text-xs font-medium transition-colors',
                  active ? 'text-primary' : 'text-muted-foreground'
                )}
              >
                <Icon className="h-5 w-5" />
                {label}
              </Link>
            </li>
          );
        })}
      </ul>
    </nav>
  );
}
