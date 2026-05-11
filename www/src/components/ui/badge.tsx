import { cn } from '@/lib/utils';
import { Slot } from '@radix-ui/react-slot';
import { type VariantProps } from 'class-variance-authority';
import * as React from 'react';

import { badgeVariants } from './badge-variants';

const Badge = React.forwardRef<
  HTMLSpanElement,
  React.ComponentProps<'span'> &
    VariantProps<typeof badgeVariants> & { asChild?: boolean }
>(({ className, variant, asChild = false, ...props }, ref) => {
  const Comp = asChild ? Slot : 'span';

  return (
    <Comp
      ref={ref}
      data-slot='badge'
      className={cn(badgeVariants({ variant }), className)}
      {...props}
    />
  );
});

Badge.displayName = 'Badge';

export { Badge };
