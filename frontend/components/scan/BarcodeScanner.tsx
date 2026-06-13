'use client';

import { useEffect } from 'react';
import { useScanner, type ScannerError } from '@/hooks/useScanner';

interface BarcodeScannerProps {
  onDetected: (barcode: string) => void;
  onError?: (error: ScannerError) => void;
}

/**
 * Full-screen camera scanner with an animated corner overlay. Starts on mount and
 * tears down the camera on unmount. Must be loaded via `next/dynamic` with
 * `ssr: false` — the underlying ZXing reader cannot run on the server.
 */
export function BarcodeScanner({ onDetected, onError }: BarcodeScannerProps) {
  const { videoRef, isScanning, error, startScanning, stopScanning } = useScanner({
    onDetected,
  });

  useEffect(() => {
    startScanning();
    return () => stopScanning();
  }, [startScanning, stopScanning]);

  useEffect(() => {
    if (error) onError?.(error);
  }, [error, onError]);

  return (
    <div className="fixed inset-0 z-40 bg-black">
      <video
        ref={videoRef}
        className="h-full w-full object-cover"
        playsInline
        muted
        autoPlay
      />

      <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
        <div className="relative h-64 w-64">
          <span className="absolute left-0 top-0 h-10 w-10 rounded-tl-lg border-l-4 border-t-4 border-primary" />
          <span className="absolute right-0 top-0 h-10 w-10 rounded-tr-lg border-r-4 border-t-4 border-primary" />
          <span className="absolute bottom-0 left-0 h-10 w-10 rounded-bl-lg border-b-4 border-l-4 border-primary" />
          <span className="absolute bottom-0 right-0 h-10 w-10 rounded-br-lg border-b-4 border-r-4 border-primary" />
        </div>
      </div>

      {isScanning && (
        <p className="absolute inset-x-0 bottom-28 animate-pulse text-center text-sm font-medium text-white">
          Scanning…
        </p>
      )}
    </div>
  );
}
