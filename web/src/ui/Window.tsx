import type { ReactNode } from 'react'

import { classNames } from './class-names.ts'

interface Props {
  children: ReactNode
  className?: string
  testId?: string
}

/** The game's menu window: bordered, dark blue gradient. */
export function Window({ children, className, testId }: Props) {
  return (
    <div className={classNames('window', className)} data-testid={testId}>
      {children}
    </div>
  )
}
